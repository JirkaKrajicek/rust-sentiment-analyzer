use std::path::Path;

use anyhow::Context;
use ndarray::Array2;
use ort::inputs;
use ort::session::Session;
use ort::value::TensorRef;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

use crate::{
    application::port::sentiment_analyzer::SentimentAnalyzer, domain::sentiment::SentimentType,
};

pub struct OnnxAnalyzer {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
}

impl OnnxAnalyzer {
    pub fn new(model_path: &Path, tokenizer_path: &Path) -> anyhow::Result<Self> {
        let session = Session::builder()?
            .commit_from_file(model_path)
            .context("Failed to load ONNX model")?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;
        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
        })
    }
}

#[async_trait::async_trait]
impl SentimentAnalyzer for OnnxAnalyzer {
    async fn analyze(&self, text: &str) -> Result<(SentimentType, f64), anyhow::Error> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();
        let seq_len = input_ids.len();

        let input_ids_array = Array2::from_shape_vec((1, seq_len), input_ids)
            .context("Failed to build input_ids tensor")?;
        let attention_mask_array = Array2::from_shape_vec((1, seq_len), attention_mask)
            .context("Failed to build attention_mask tensor")?;

        let (neg, pos) = tokio::task::block_in_place(|| -> anyhow::Result<(f32, f32)> {
            let mut session = self.session.blocking_lock();
            let outputs = session.run(inputs![
                "input_ids" => TensorRef::<i64>::from_array_view(&input_ids_array)?,
                "attention_mask" => TensorRef::<i64>::from_array_view(&attention_mask_array)?,
            ])?;
            let (_, logits) = outputs["logits"].try_extract_tensor::<f32>()?;
            Ok((logits[0], logits[1]))
        })?;

        // Numerically stable softmax over [neg, pos]
        let max = neg.max(pos);
        let exp_neg = (neg - max).exp();
        let exp_pos = (pos - max).exp();
        let prob_pos = (exp_pos / (exp_neg + exp_pos)) as f64;

        Ok(classify_probability(prob_pos))
    }
}

fn classify_probability(prob_pos: f64) -> (SentimentType, f64) {
    if prob_pos >= 0.5 {
        (SentimentType::Positive, prob_pos)
    } else {
        (SentimentType::Negative, 1.0 - prob_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::classify_probability;
    use crate::domain::sentiment::SentimentType;

    #[test]
    fn classifies_positive_probability_including_a_tie() {
        assert_eq!(classify_probability(0.5), (SentimentType::Positive, 0.5));
        assert_eq!(classify_probability(0.9), (SentimentType::Positive, 0.9));
    }

    #[test]
    fn classifies_negative_probability_as_negative_confidence() {
        assert_eq!(classify_probability(0.49), (SentimentType::Negative, 0.51));
    }
}

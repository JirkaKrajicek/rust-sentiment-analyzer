use std::path::Path;

use anyhow::Context;
use ndarray::Array2;
use ort::{Session, inputs};
use tokenizers::Tokenizer;

use crate::{
    application::port::sentiment_analyzer::SentimentAnalyzer, domain::sentiment::SentimentType,
};

pub struct OnnxAnalyzer {
    session: Session,
    tokenizer: Tokenizer,
}

impl OnnxAnalyzer {
    pub fn new(model_path: &Path, tokenizer_path: &Path) -> anyhow::Result<Self> {
        let session = Session::builder()?
            .commit_from_file(model_path)
            .context("Failed to load ONNX model")?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;
        Ok(Self { session, tokenizer })
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
            let outputs = self.session.run(inputs![
                "input_ids" => input_ids_array.view(),
                "attention_mask" => attention_mask_array.view(),
            ]?)?;
            let logits = outputs["logits"].try_extract_tensor::<f32>()?;
            let view = logits.view();
            Ok((view[[0, 0]], view[[0, 1]]))
        })?;

        // Numerically stable softmax over [neg, pos]
        let max = neg.max(pos);
        let exp_neg = (neg - max).exp();
        let exp_pos = (pos - max).exp();
        let prob_pos = (exp_pos / (exp_neg + exp_pos)) as f64;

        let (sentiment, probability) = if prob_pos > 0.6 {
            (SentimentType::Positive, prob_pos)
        } else if prob_pos >= 0.4 && prob_pos <= 0.6 {
            (SentimentType::Neutral, prob_pos)
        } else {
            (SentimentType::Negative, 1.0 - prob_pos)
        };

        Ok((sentiment, probability))
    }
}

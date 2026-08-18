use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Context;
use ndarray::Array2;
use ort::inputs;
use ort::session::Session;
use ort::value::TensorRef;
use tokenizers::{Tokenizer, TruncationParams};
use tokio::{sync::Semaphore, time::timeout};

use crate::{
    application::port::sentiment_analyzer::{InferenceError, SentimentAnalyzer},
    domain::sentiment::SentimentType,
};

pub struct OnnxAnalyzer {
    session: Arc<Mutex<Session>>,
    tokenizer: Tokenizer,
    inference_slots: Arc<Semaphore>,
    queue_timeout: Duration,
    execution_timeout: Duration,
}

impl OnnxAnalyzer {
    pub fn new(
        model_path: &Path,
        tokenizer_path: &Path,
        max_tokens: usize,
        queue_timeout: Duration,
        execution_timeout: Duration,
    ) -> anyhow::Result<Self> {
        let mut session = Session::builder()?
            .commit_from_file(model_path)
            .context("Failed to load ONNX model")?;
        let mut tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: max_tokens,
                ..Default::default()
            }))
            .map_err(|e| anyhow::anyhow!("Failed to configure tokenizer truncation: {e}"))?;
        verify_session(&tokenizer, &mut session)?;
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            tokenizer,
            inference_slots: Arc::new(Semaphore::new(1)),
            queue_timeout,
            execution_timeout,
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

        let permit = timeout(
            self.queue_timeout,
            Arc::clone(&self.inference_slots).acquire_owned(),
        )
        .await
        .map_err(|_| InferenceError::Overloaded)?
        .map_err(|_| InferenceError::WorkerFailed)?;
        let session = Arc::clone(&self.session);
        let inference = tokio::task::spawn_blocking(move || -> anyhow::Result<(f32, f32)> {
            let _permit = permit;
            let mut session = session
                .lock()
                .map_err(|_| anyhow::anyhow!("Inference session lock was poisoned"))?;
            let outputs = session.run(inputs![
                "input_ids" => TensorRef::<i64>::from_array_view(&input_ids_array)?,
                "attention_mask" => TensorRef::<i64>::from_array_view(&attention_mask_array)?,
            ])?;
            let (_, logits) = outputs["logits"].try_extract_tensor::<f32>()?;
            Ok((logits[0], logits[1]))
        });
        let (neg, pos) = timeout(self.execution_timeout, inference)
            .await
            .map_err(|_| InferenceError::TimedOut)?
            .map_err(|_| InferenceError::WorkerFailed)??;

        // Numerically stable softmax over [neg, pos]
        let max = neg.max(pos);
        let exp_neg = (neg - max).exp();
        let exp_pos = (pos - max).exp();
        let prob_pos = (exp_pos / (exp_neg + exp_pos)) as f64;

        Ok(classify_probability(prob_pos))
    }

    fn chunk_text(&self, text: &str) -> Result<Vec<String>, anyhow::Error> {
        let mut remaining = text.trim();
        let mut chunks = Vec::new();
        while !remaining.is_empty() {
            let encoding = self
                .tokenizer
                .encode(remaining, true)
                .map_err(|error| anyhow::anyhow!("Document tokenization failed: {error}"))?;
            let end = encoding
                .get_offsets()
                .iter()
                .map(|&(_, end)| end)
                .max()
                .unwrap_or(0);
            if end == 0 {
                anyhow::bail!("Document tokenization did not advance");
            }
            chunks.push(remaining[..end].trim().to_string());
            remaining = remaining[end..].trim_start();
        }
        Ok(chunks)
    }

    async fn is_ready(&self) -> Result<(), anyhow::Error> {
        if self.session.is_poisoned() {
            anyhow::bail!("Inference session lock was poisoned");
        }
        Ok(())
    }
}

fn verify_session(tokenizer: &Tokenizer, session: &mut Session) -> anyhow::Result<()> {
    let encoding = tokenizer
        .encode("readiness probe", true)
        .map_err(|error| anyhow::anyhow!("Readiness probe tokenization failed: {error}"))?;
    let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| i64::from(id)).collect();
    let attention_mask: Vec<i64> = encoding
        .get_attention_mask()
        .iter()
        .map(|&mask| i64::from(mask))
        .collect();
    let sequence_length = input_ids.len();
    let input_ids = Array2::from_shape_vec((1, sequence_length), input_ids)
        .context("Failed to create readiness probe input_ids tensor")?;
    let attention_mask = Array2::from_shape_vec((1, sequence_length), attention_mask)
        .context("Failed to create readiness probe attention_mask tensor")?;
    let outputs = session.run(inputs![
        "input_ids" => TensorRef::<i64>::from_array_view(&input_ids)?,
        "attention_mask" => TensorRef::<i64>::from_array_view(&attention_mask)?,
    ])?;
    let (_, logits) = outputs["logits"].try_extract_tensor::<f32>()?;
    if logits.len() < 2 {
        anyhow::bail!("Readiness probe expected two sentiment logits");
    }
    Ok(())
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

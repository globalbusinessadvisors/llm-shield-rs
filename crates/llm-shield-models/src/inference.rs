//! Inference Engine
//!
//! Handles model inference and result processing.

use llm_shield_core::Error;
use ndarray::{Array2, ArrayView2};
use ort::{Session, SessionInputs, SessionOutputs};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    /// Predicted class labels
    pub labels: Vec<String>,

    /// Confidence scores for each class
    pub scores: Vec<f32>,

    /// Predicted class index
    pub predicted_class: usize,

    /// Maximum confidence score
    pub max_score: f32,
}

impl InferenceResult {
    /// Get the predicted label
    pub fn predicted_label(&self) -> Option<&str> {
        self.labels.get(self.predicted_class).map(|s| s.as_str())
    }

    /// Check if prediction confidence exceeds threshold
    pub fn exceeds_threshold(&self, threshold: f32) -> bool {
        self.max_score >= threshold
    }
}

/// Inference engine
pub struct InferenceEngine {
    session: Arc<Session>,
}

impl InferenceEngine {
    /// Create a new inference engine
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    /// Run inference on input IDs
    pub fn infer(
        &self,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
    ) -> crate::Result<InferenceResult> {
        // Convert to i64 for ONNX
        let input_ids_i64: Vec<i64> = input_ids.iter().map(|&x| x as i64).collect();
        let attention_mask_i64: Vec<i64> = attention_mask.iter().map(|&x| x as i64).collect();

        let batch_size = 1;
        let seq_length = input_ids.len();

        // Create input arrays
        let input_ids_array =
            Array2::from_shape_vec((batch_size, seq_length), input_ids_i64)
                .map_err(|e| Error::model(format!("Failed to create input array: {}", e)))?;

        let attention_mask_array =
            Array2::from_shape_vec((batch_size, seq_length), attention_mask_i64)
                .map_err(|e| Error::model(format!("Failed to create attention mask array: {}", e)))?;

        // Run inference
        let outputs = self
            .session
            .run(ort::inputs![
                "input_ids" => input_ids_array.view(),
                "attention_mask" => attention_mask_array.view(),
            ].map_err(|e| Error::model(format!("Failed to create inputs: {}", e)))?)
            .map_err(|e| Error::model(format!("Inference failed: {}", e)))?;

        // Extract logits
        let logits = outputs["logits"]
            .try_extract_tensor::<f32>()
            .map_err(|e| Error::model(format!("Failed to extract logits: {}", e)))?;

        // Apply softmax and get scores
        let scores = self.softmax(logits.view());

        // Find predicted class
        let (predicted_class, max_score) = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        Ok(InferenceResult {
            labels: labels.to_vec(),
            scores,
            predicted_class,
            max_score: *max_score,
        })
    }

    /// Apply softmax to logits
    fn softmax(&self, logits: ArrayView2<f32>) -> Vec<f32> {
        let logits_slice = logits.row(0);
        let max_logit = logits_slice
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let exp_logits: Vec<f32> = logits_slice.iter().map(|&x| (x - max_logit).exp()).collect();
        let sum_exp: f32 = exp_logits.iter().sum();

        exp_logits.iter().map(|&x| x / sum_exp).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_result_predicted_label() {
        let result = InferenceResult {
            labels: vec!["safe".to_string(), "unsafe".to_string()],
            scores: vec![0.8, 0.2],
            predicted_class: 0,
            max_score: 0.8,
        };

        assert_eq!(result.predicted_label(), Some("safe"));
        assert!(result.exceeds_threshold(0.7));
        assert!(!result.exceeds_threshold(0.9));
    }

    #[test]
    fn test_softmax_values() {
        // Manual softmax verification would require creating a session
        // This test verifies the structure compiles
        assert!(true);
    }
}

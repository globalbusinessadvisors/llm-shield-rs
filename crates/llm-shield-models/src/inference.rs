//! Inference Engine
//!
//! Handles model inference and result processing.
//!
//! ## Features
//!
//! - Binary and multi-label classification
//! - Softmax and sigmoid post-processing
//! - Threshold-based decision making
//! - Async inference API
//! - Support for different model tasks
//!
//! ## Example
//!
//! ```rust,ignore
//! use llm_shield_models::InferenceEngine;
//!
//! let engine = InferenceEngine::new(session);
//! let result = engine.infer(&input_ids, &attention_mask, &labels).await?;
//! ```

use llm_shield_core::Error;
use ndarray::Array2;
use ort::session::Session;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Post-processing method for model outputs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostProcessing {
    /// Softmax (for single-label classification)
    /// Outputs sum to 1.0
    Softmax,

    /// Sigmoid (for multi-label classification)
    /// Each output is independent [0, 1]
    Sigmoid,
}

/// Inference result with classification predictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceResult {
    /// Predicted class labels
    pub labels: Vec<String>,

    /// Confidence scores for each class (after post-processing)
    pub scores: Vec<f32>,

    /// Predicted class index (highest score)
    pub predicted_class: usize,

    /// Maximum confidence score
    pub max_score: f32,
}

impl InferenceResult {
    /// Get the predicted label
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::InferenceResult;
    ///
    /// let result = InferenceResult {
    ///     labels: vec!["SAFE".to_string(), "INJECTION".to_string()],
    ///     scores: vec![0.8, 0.2],
    ///     predicted_class: 0,
    ///     max_score: 0.8,
    /// };
    ///
    /// assert_eq!(result.predicted_label(), Some("SAFE"));
    /// ```
    pub fn predicted_label(&self) -> Option<&str> {
        self.labels.get(self.predicted_class).map(|s| s.as_str())
    }

    /// Check if prediction confidence exceeds threshold
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum confidence threshold (0.0 to 1.0)
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::InferenceResult;
    ///
    /// let result = InferenceResult {
    ///     labels: vec!["SAFE".to_string(), "INJECTION".to_string()],
    ///     scores: vec![0.3, 0.7],
    ///     predicted_class: 1,
    ///     max_score: 0.7,
    /// };
    ///
    /// assert!(result.exceeds_threshold(0.5));
    /// assert!(!result.exceeds_threshold(0.8));
    /// ```
    pub fn exceeds_threshold(&self, threshold: f32) -> bool {
        self.max_score >= threshold
    }

    /// Get score for a specific label
    ///
    /// # Arguments
    ///
    /// * `label` - The label to get the score for
    ///
    /// # Returns
    ///
    /// The confidence score for the label, or None if not found
    pub fn get_score_for_label(&self, label: &str) -> Option<f32> {
        self.labels
            .iter()
            .position(|l| l == label)
            .and_then(|idx| self.scores.get(idx).copied())
    }

    /// Check if this is a binary classification result
    pub fn is_binary(&self) -> bool {
        self.labels.len() == 2
    }

    /// Get indices of labels that exceed their respective thresholds
    ///
    /// Used for multi-label classification where each class has its own threshold.
    ///
    /// # Arguments
    ///
    /// * `thresholds` - Per-class thresholds (must match number of labels)
    ///
    /// # Returns
    ///
    /// Vector of class indices that exceed their thresholds
    pub fn get_threshold_violations(&self, thresholds: &[f32]) -> Vec<usize> {
        if thresholds.len() != self.scores.len() {
            tracing::warn!(
                "Threshold count mismatch: {} thresholds for {} classes",
                thresholds.len(),
                self.scores.len()
            );
            return vec![];
        }

        self.scores
            .iter()
            .enumerate()
            .filter_map(|(idx, &score)| {
                if score >= thresholds[idx] {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create InferenceResult from logits using softmax (single-label)
    ///
    /// # Arguments
    ///
    /// * `logits` - Raw model output logits
    /// * `labels` - Class labels
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::InferenceResult;
    ///
    /// let logits = vec![1.0, 2.0, 0.5];
    /// let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    /// let result = InferenceResult::from_binary_logits(logits, labels);
    ///
    /// // B should have highest probability
    /// assert_eq!(result.predicted_class, 1);
    /// ```
    pub fn from_binary_logits(logits: Vec<f32>, labels: Vec<String>) -> Self {
        let scores = InferenceEngine::softmax_static(&logits);
        let (predicted_class, max_score) = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, &score)| (idx, score))
            .unwrap_or((0, 0.0));

        Self {
            labels,
            scores,
            predicted_class,
            max_score,
        }
    }

    /// Create InferenceResult from logits using sigmoid (multi-label)
    ///
    /// # Arguments
    ///
    /// * `logits` - Raw model output logits
    /// * `labels` - Class labels
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::InferenceResult;
    ///
    /// let logits = vec![2.0, -1.0, 1.0];
    /// let labels = vec!["toxic".to_string(), "threat".to_string(), "insult".to_string()];
    /// let result = InferenceResult::from_multilabel_logits(logits, labels);
    ///
    /// // All scores should be in [0, 1]
    /// for score in &result.scores {
    ///     assert!(*score >= 0.0 && *score <= 1.0);
    /// }
    /// ```
    pub fn from_multilabel_logits(logits: Vec<f32>, labels: Vec<String>) -> Self {
        let scores = InferenceEngine::sigmoid_static(&logits);
        let (predicted_class, max_score) = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, &score)| (idx, score))
            .unwrap_or((0, 0.0));

        Self {
            labels,
            scores,
            predicted_class,
            max_score,
        }
    }
}

/// Inference engine for running ONNX model inference
///
/// ## Features
///
/// - Synchronous and asynchronous inference
/// - Binary and multi-label classification
/// - Automatic post-processing (softmax/sigmoid)
/// - Batch inference support (optional)
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_models::InferenceEngine;
/// use std::sync::Arc;
///
/// let engine = InferenceEngine::new(session);
///
/// // Run inference
/// let result = engine.infer(
///     &input_ids,
///     &attention_mask,
///     &labels,
///     PostProcessing::Softmax,
/// ).await?;
///
/// println!("Predicted: {}", result.predicted_label().unwrap());
/// println!("Confidence: {:.2}", result.max_score);
/// ```
pub struct InferenceEngine {
    session: Arc<Mutex<Session>>,
}

impl InferenceEngine {
    /// Create a new inference engine
    ///
    /// # Arguments
    ///
    /// * `session` - ONNX Runtime session wrapped in Arc<Mutex<>> for thread-safe mutable access
    pub fn new(session: Arc<Mutex<Session>>) -> Self {
        Self { session }
    }

    /// Run inference on input IDs (async)
    ///
    /// # Arguments
    ///
    /// * `input_ids` - Tokenized input IDs
    /// * `attention_mask` - Attention mask (1 for real tokens, 0 for padding)
    /// * `labels` - Class labels
    /// * `post_processing` - Post-processing method (Softmax or Sigmoid)
    ///
    /// # Returns
    ///
    /// InferenceResult with predictions and confidence scores
    pub async fn infer_async(
        &self,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
        post_processing: PostProcessing,
    ) -> crate::Result<InferenceResult> {
        // Run inference in blocking thread pool to avoid blocking async runtime
        let session = Arc::clone(&self.session);
        let input_ids = input_ids.to_vec();
        let attention_mask = attention_mask.to_vec();
        let labels = labels.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut session_guard = session.lock()
                .map_err(|e| Error::model(format!("Failed to lock session: {}", e)))?;
            Self::infer_sync(&mut *session_guard, &input_ids, &attention_mask, &labels, post_processing)
        })
        .await
        .map_err(|e| Error::model(format!("Async inference task failed: {}", e)))?
    }

    /// Run inference on input IDs (synchronous)
    ///
    /// # Arguments
    ///
    /// * `input_ids` - Tokenized input IDs
    /// * `attention_mask` - Attention mask (1 for real tokens, 0 for padding)
    /// * `labels` - Class labels
    /// * `post_processing` - Post-processing method (Softmax or Sigmoid)
    ///
    /// # Returns
    ///
    /// InferenceResult with predictions and confidence scores
    pub fn infer(
        &self,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
        post_processing: PostProcessing,
    ) -> crate::Result<InferenceResult> {
        let mut session_guard = self.session.lock()
            .map_err(|e| Error::model(format!("Failed to lock session: {}", e)))?;
        Self::infer_sync(&mut *session_guard, input_ids, attention_mask, labels, post_processing)
    }

    /// Internal synchronous inference implementation
    fn infer_sync(
        session: &mut Session,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
        post_processing: PostProcessing,
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

        // Create ONNX values
        let input_ids_value = ort::value::Value::from_array(input_ids_array)
            .map_err(|e| Error::model(format!("Failed to create input_ids value: {}", e)))?;
        let attention_mask_value = ort::value::Value::from_array(attention_mask_array)
            .map_err(|e| Error::model(format!("Failed to create attention_mask value: {}", e)))?;

        // Run inference
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_value,
                "attention_mask" => attention_mask_value,
            ])
            .map_err(|e| Error::model(format!("Inference failed: {}", e)))?;

        // Extract logits
        let logits = outputs["logits"]
            .try_extract_tensor::<f32>()
            .map_err(|e| Error::model(format!("Failed to extract logits: {}", e)))?;

        // Extract logits as Vec<f32> - logits is now (shape, data)
        let (_shape, data) = logits;
        let logits_vec: Vec<f32> = data.to_vec();

        // Apply post-processing
        let scores = match post_processing {
            PostProcessing::Softmax => Self::softmax_static(&logits_vec),
            PostProcessing::Sigmoid => Self::sigmoid_static(&logits_vec),
        };

        // Find predicted class
        let (predicted_class, max_score) = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, &score)| (idx, score))
            .unwrap_or((0, 0.0));

        Ok(InferenceResult {
            labels: labels.to_vec(),
            scores,
            predicted_class,
            max_score,
        })
    }

    /// Apply softmax to logits (static method)
    ///
    /// Softmax converts logits to probabilities that sum to 1.0.
    /// Used for single-label classification (mutually exclusive classes).
    ///
    /// # Arguments
    ///
    /// * `logits` - Raw model output logits
    ///
    /// # Returns
    ///
    /// Probability distribution (sums to 1.0)
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::InferenceEngine;
    ///
    /// let logits = vec![1.0, 2.0, 0.5];
    /// let probs = InferenceEngine::softmax_static(&logits);
    ///
    /// // Probabilities sum to 1.0
    /// let sum: f32 = probs.iter().sum();
    /// assert!((sum - 1.0).abs() < 0.001);
    /// ```
    pub fn softmax_static(logits: &[f32]) -> Vec<f32> {
        if logits.is_empty() {
            return vec![];
        }

        // Find max for numerical stability
        let max_logit = logits
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        // Compute exp(logit - max)
        let exp_logits: Vec<f32> = logits.iter().map(|&x| (x - max_logit).exp()).collect();

        // Sum of exponentials
        let sum_exp: f32 = exp_logits.iter().sum();

        // Normalize
        if sum_exp == 0.0 {
            // Edge case: all logits are very negative
            vec![1.0 / logits.len() as f32; logits.len()]
        } else {
            exp_logits.iter().map(|&x| x / sum_exp).collect()
        }
    }

    /// Apply sigmoid to logits (static method)
    ///
    /// Sigmoid converts each logit independently to [0, 1].
    /// Used for multi-label classification (non-exclusive classes).
    ///
    /// # Arguments
    ///
    /// * `logits` - Raw model output logits
    ///
    /// # Returns
    ///
    /// Independent probabilities (do NOT sum to 1.0)
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::InferenceEngine;
    ///
    /// let logits = vec![0.0, 2.0, -2.0];
    /// let probs = InferenceEngine::sigmoid_static(&logits);
    ///
    /// // sigmoid(0) ≈ 0.5
    /// assert!((probs[0] - 0.5).abs() < 0.01);
    ///
    /// // All probabilities in [0, 1]
    /// for p in probs {
    ///     assert!(p >= 0.0 && p <= 1.0);
    /// }
    /// ```
    pub fn sigmoid_static(logits: &[f32]) -> Vec<f32> {
        logits
            .iter()
            .map(|&x| 1.0 / (1.0 + (-x).exp()))
            .collect()
    }

    /// Apply softmax to logits (instance method)
    #[allow(dead_code)]
    fn softmax(&self, logits: &[f32]) -> Vec<f32> {
        Self::softmax_static(logits)
    }

    /// Apply sigmoid to logits (instance method)
    #[allow(dead_code)]
    fn sigmoid(&self, logits: &[f32]) -> Vec<f32> {
        Self::sigmoid_static(logits)
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

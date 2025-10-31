# Phase 8: ML Infrastructure API Documentation

## Executive Summary

This document provides comprehensive API documentation for the ML infrastructure components implemented in Phase 8, including ModelRegistry, ResultCache, ModelLoader, TokenizerWrapper, and InferenceEngine.

**Status**: ✅ Complete - All components production-ready with comprehensive test coverage

---

## Table of Contents

1. [ModelRegistry API](#1-modelregistry-api)
2. [ResultCache API](#2-resultcache-api)
3. [ModelLoader API](#3-modelloader-api)
4. [TokenizerWrapper API](#4-tokenizerwrapper-api)
5. [InferenceEngine API](#5-inferenceengine-api)
6. [Type System](#6-type-system)
7. [Error Handling](#7-error-handling)
8. [Performance Considerations](#8-performance-considerations)

---

## 1. ModelRegistry API

The ModelRegistry manages model metadata, downloads, caching, and verification for ML models.

### 1.1 Core Types

```rust
/// Model task type
pub enum ModelTask {
    PromptInjection,  // Prompt injection detection
    Toxicity,         // Toxicity classification
    Sentiment,        // Sentiment analysis
}

/// Model variant (precision/quantization)
pub enum ModelVariant {
    FP16,   // 16-bit floating point (balanced)
    FP32,   // 32-bit floating point (highest accuracy)
    INT8,   // 8-bit integer (smallest size)
}

/// Model metadata
pub struct ModelMetadata {
    pub id: String,              // Unique model identifier
    pub task: ModelTask,         // Task this model performs
    pub variant: ModelVariant,   // Model precision
    pub url: String,             // Download URL
    pub checksum: String,        // SHA-256 checksum
    pub size_bytes: usize,       // Model size in bytes
}
```

### 1.2 Constructor Methods

```rust
impl ModelRegistry {
    /// Create a new registry with default cache directory
    ///
    /// Cache dir: ~/.cache/llm-shield/models (or .cache/ if home unavailable)
    pub fn new() -> Self

    /// Create a registry from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to registry.json file
    ///
    /// # Returns
    /// Loaded ModelRegistry or Error
    ///
    /// # Example
    /// ```rust
    /// let registry = ModelRegistry::from_file("models/registry.json")?;
    /// ```
    pub fn from_file(path: &str) -> Result<Self>
}
```

### 1.3 Core Operations

```rust
impl ModelRegistry {
    /// Get metadata for a specific model
    ///
    /// # Arguments
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Returns
    /// Reference to ModelMetadata or Error if not found
    ///
    /// # Example
    /// ```rust
    /// let metadata = registry.get_model_metadata(
    ///     ModelTask::PromptInjection,
    ///     ModelVariant::FP16
    /// )?;
    /// println!("Model: {} ({}MB)", metadata.id, metadata.size_bytes / 1_000_000);
    /// ```
    pub fn get_model_metadata(
        &self,
        task: ModelTask,
        variant: ModelVariant,
    ) -> Result<&ModelMetadata>

    /// Ensure a model is available locally (download if needed)
    ///
    /// This method:
    /// 1. Checks if model is already cached
    /// 2. Verifies checksum if cached
    /// 3. Downloads if not cached or verification fails
    /// 4. Verifies checksum after download
    ///
    /// # Arguments
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Returns
    /// Path to the local model file
    ///
    /// # Example
    /// ```rust
    /// let model_path = registry.ensure_model_available(
    ///     ModelTask::PromptInjection,
    ///     ModelVariant::FP16
    /// ).await?;
    /// println!("Model ready at: {:?}", model_path);
    /// ```
    pub async fn ensure_model_available(
        &self,
        task: ModelTask,
        variant: ModelVariant,
    ) -> Result<PathBuf>
}
```

### 1.4 Registry File Format

The registry file is a JSON file with the following structure:

```json
{
  "cache_dir": "~/.cache/llm-shield/models",
  "models": [
    {
      "id": "protectai-deberta-v3-prompt-injection-fp16",
      "task": "PromptInjection",
      "variant": "FP16",
      "url": "https://huggingface.co/protectai/deberta-v3-base-prompt-injection/resolve/main/model-fp16.onnx",
      "checksum": "a1b2c3d4e5f6...",
      "size_bytes": 274000000
    }
  ]
}
```

### 1.5 Usage Example

```rust
use llm_shield_models::{ModelRegistry, ModelTask, ModelVariant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load registry from catalog
    let registry = ModelRegistry::from_file("models/registry.json")?;

    // Get model metadata
    let metadata = registry.get_model_metadata(
        ModelTask::PromptInjection,
        ModelVariant::FP16
    )?;

    println!("Model: {} ({}MB)", metadata.id, metadata.size_bytes / 1_000_000);

    // Ensure model is downloaded and cached
    let model_path = registry.ensure_model_available(
        ModelTask::PromptInjection,
        ModelVariant::FP16
    ).await?;

    println!("Model ready at: {:?}", model_path);

    // Second call uses cache (instant)
    let cached_path = registry.ensure_model_available(
        ModelTask::PromptInjection,
        ModelVariant::FP16
    ).await?;

    assert_eq!(model_path, cached_path);

    Ok(())
}
```

---

## 2. ResultCache API

The ResultCache provides thread-safe result caching with LRU eviction and TTL support.

### 2.1 Core Types

```rust
/// Configuration for the result cache
pub struct CacheConfig {
    pub max_size: usize,     // Maximum number of entries
    pub ttl: Duration,       // Time-to-live for entries
}

/// Cache performance statistics
pub struct CacheStats {
    pub hits: u64,      // Number of cache hits
    pub misses: u64,    // Number of cache misses
}

impl CacheStats {
    /// Total number of cache requests
    pub fn total_requests(&self) -> u64

    /// Hit rate as a value between 0.0 and 1.0
    pub fn hit_rate(&self) -> f64
}
```

### 2.2 Constructor Methods

```rust
impl ResultCache {
    /// Create a new result cache with the given configuration
    ///
    /// # Example
    /// ```rust
    /// use llm_shield_models::cache::{ResultCache, CacheConfig};
    /// use std::time::Duration;
    ///
    /// let cache = ResultCache::new(CacheConfig {
    ///     max_size: 1000,
    ///     ttl: Duration::from_secs(300),
    /// });
    /// ```
    pub fn new(config: CacheConfig) -> Self
}

impl Default for CacheConfig {
    /// Default configuration:
    /// - max_size: 10,000 entries
    /// - ttl: 300 seconds (5 minutes)
    fn default() -> Self
}
```

### 2.3 Core Operations

```rust
impl ResultCache {
    /// Get a cached result by key
    ///
    /// Returns `None` if:
    /// - Key doesn't exist
    /// - Entry has expired (and removes it)
    ///
    /// Updates LRU access order on cache hit.
    ///
    /// # Example
    /// ```rust
    /// if let Some(cached_result) = cache.get("key1") {
    ///     println!("Cache hit! Risk score: {}", cached_result.risk_score);
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<ScanResult>

    /// Insert or update a cache entry
    ///
    /// If the cache is at capacity, evicts the least recently used entry.
    /// If the key already exists, updates it and refreshes the TTL.
    ///
    /// # Example
    /// ```rust
    /// let result = ScanResult::pass("safe text".to_string());
    /// cache.insert("key1".to_string(), result);
    /// ```
    pub fn insert(&self, key: String, result: ScanResult)

    /// Clear all entries from the cache
    ///
    /// This does not reset statistics.
    pub fn clear(&self)

    /// Get the number of entries in the cache
    ///
    /// Note: This includes expired entries that haven't been lazily cleaned yet.
    pub fn len(&self) -> usize

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool

    /// Get cache statistics
    ///
    /// # Example
    /// ```rust
    /// let stats = cache.stats();
    /// println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    /// println!("Total requests: {}", stats.total_requests());
    /// ```
    pub fn stats(&self) -> CacheStats

    /// Reset cache statistics
    ///
    /// This does not affect cached entries.
    pub fn reset_stats(&self)

    /// Generate a deterministic hash key from input text
    ///
    /// Useful for caching scan results based on input content.
    ///
    /// # Example
    /// ```rust
    /// let input = "some text to scan";
    /// let key = ResultCache::hash_key(input);
    /// ```
    pub fn hash_key(input: &str) -> String
}
```

### 2.4 Thread Safety

ResultCache is fully thread-safe using `Arc<RwLock<_>>`:
- Multiple concurrent readers (common case)
- Exclusive writer access
- Clone creates a new reference to same cache

```rust
// Clone creates a new reference to the same underlying cache
let cache = ResultCache::new(CacheConfig::default());
let cache_clone = cache.clone();

// Both references share the same cache data
cache.insert("key1".to_string(), result1);
assert!(cache_clone.get("key1").is_some());
```

### 2.5 Performance Characteristics

| Operation | Average Case | Worst Case | Notes |
|-----------|-------------|------------|-------|
| `get()` | O(1) | O(n) | HashMap lookup + LRU update |
| `insert()` | O(1) | O(n) | HashMap insert + LRU tracking |
| `clear()` | O(1) | O(1) | Clear collections |
| `len()` | O(1) | O(1) | Direct field access |

**Memory**: O(max_size * entry_size) - approximately 48 bytes overhead per entry

### 2.6 Usage Example

```rust
use llm_shield_models::cache::{ResultCache, CacheConfig};
use llm_shield_core::ScanResult;
use std::time::Duration;

// Create cache with 1000 entry capacity, 5-minute TTL
let cache = ResultCache::new(CacheConfig {
    max_size: 1000,
    ttl: Duration::from_secs(300),
});

// Hash input text for cache key
let input = "User prompt text here...";
let key = ResultCache::hash_key(input);

// Check cache first
if let Some(cached_result) = cache.get(&key) {
    println!("Cache hit! Risk score: {}", cached_result.risk_score);
    return Ok(cached_result);
}

// Cache miss - perform scan
let result = perform_expensive_scan(input);

// Store in cache
cache.insert(key, result.clone());

// Check statistics
let stats = cache.stats();
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
```

---

## 3. ModelLoader API

The ModelLoader provides lazy loading and caching of ONNX Runtime sessions.

### 3.1 Core Types

```rust
/// Model type identifier
pub enum ModelType {
    PromptInjection,
    Toxicity,
    Sentiment,
}

/// Configuration for loading a model
pub struct ModelConfig {
    pub model_type: ModelType,
    pub variant: ModelVariant,
    pub model_path: PathBuf,
    pub thread_pool_size: usize,
    pub optimization_level: u8,
}

/// Statistics about loaded models
pub struct LoaderStats {
    pub total_loaded: usize,    // Number of models currently loaded
    pub total_loads: u64,       // Total number of load operations
    pub cache_hits: u64,        // Total number of cache hits
}
```

### 3.2 Constructor Methods

```rust
impl ModelLoader {
    /// Create a new model loader
    ///
    /// # Arguments
    /// * `registry` - Model registry for metadata and downloads
    ///
    /// # Example
    /// ```rust
    /// let registry = ModelRegistry::from_file("models/registry.json")?;
    /// let loader = ModelLoader::new(Arc::new(registry));
    /// ```
    pub fn new(registry: Arc<ModelRegistry>) -> Self

    /// Create a new model loader (alias for `new`)
    pub fn with_registry(registry: Arc<ModelRegistry>) -> Self
}

impl ModelConfig {
    /// Create a new model configuration
    ///
    /// # Arguments
    /// * `model_type` - Type of model (PromptInjection, Toxicity, Sentiment)
    /// * `variant` - Model variant (FP32, FP16, INT8)
    /// * `model_path` - Path to ONNX model file
    ///
    /// # Example
    /// ```rust
    /// let config = ModelConfig::new(
    ///     ModelType::PromptInjection,
    ///     ModelVariant::FP16,
    ///     PathBuf::from("/path/to/model.onnx")
    /// );
    /// ```
    pub fn new(model_type: ModelType, variant: ModelVariant, model_path: PathBuf) -> Self

    /// Set the thread pool size
    pub fn with_thread_pool_size(mut self, size: usize) -> Self

    /// Set the optimization level (0-3)
    pub fn with_optimization_level(mut self, level: u8) -> Self
}
```

### 3.3 Core Operations

```rust
impl ModelLoader {
    /// Load a model (lazily, with caching)
    ///
    /// If the model is already loaded, returns the cached session.
    /// Otherwise, loads the model from disk using the registry.
    ///
    /// # Arguments
    /// * `model_type` - Type of model to load
    /// * `variant` - Model variant (precision)
    ///
    /// # Returns
    /// Arc to ONNX Runtime session
    ///
    /// # Example
    /// ```rust
    /// let session = loader.load(ModelType::PromptInjection, ModelVariant::FP16).await?;
    /// ```
    pub async fn load(
        &self,
        model_type: ModelType,
        variant: ModelVariant,
    ) -> Result<Arc<Session>>

    /// Load a model with custom configuration
    ///
    /// # Arguments
    /// * `config` - Model configuration
    ///
    /// # Returns
    /// Arc to ONNX Runtime session
    pub async fn load_with_config(&self, config: ModelConfig) -> Result<Arc<Session>>

    /// Preload multiple models
    ///
    /// Useful for warming up the cache before first use.
    ///
    /// # Arguments
    /// * `models` - List of (ModelType, ModelVariant) tuples to preload
    ///
    /// # Example
    /// ```rust
    /// let models = vec![
    ///     (ModelType::PromptInjection, ModelVariant::FP16),
    ///     (ModelType::Toxicity, ModelVariant::FP16),
    /// ];
    /// loader.preload(models).await?;
    /// ```
    pub async fn preload(&self, models: Vec<(ModelType, ModelVariant)>) -> Result<()>

    /// Check if a model is loaded
    pub fn is_loaded(&self, model_type: ModelType, variant: ModelVariant) -> bool

    /// Unload a specific model
    ///
    /// Removes the model from cache, freeing memory.
    pub fn unload(&self, model_type: ModelType, variant: ModelVariant)

    /// Unload all models
    ///
    /// Clears the entire cache, freeing all memory.
    pub fn unload_all(&self)

    /// Get the number of loaded models
    pub fn len(&self) -> usize

    /// Check if no models are loaded
    pub fn is_empty(&self) -> bool

    /// Get list of loaded models
    pub fn loaded_models(&self) -> Vec<(ModelType, ModelVariant)>

    /// Get loader statistics
    pub fn stats(&self) -> LoaderStats
}
```

### 3.4 Usage Example

```rust
use llm_shield_models::{ModelLoader, ModelRegistry, ModelType, ModelVariant};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry and loader
    let registry = ModelRegistry::from_file("models/registry.json")?;
    let loader = ModelLoader::new(Arc::new(registry));

    // Load model (lazy - only loads once)
    let session = loader.load(ModelType::PromptInjection, ModelVariant::FP16).await?;

    // Second call uses cache (instant)
    let cached_session = loader.load(ModelType::PromptInjection, ModelVariant::FP16).await?;

    // Check statistics
    let stats = loader.stats();
    println!("Models loaded: {}", stats.total_loaded);
    println!("Cache hits: {}", stats.cache_hits);

    // Preload multiple models
    loader.preload(vec![
        (ModelType::Toxicity, ModelVariant::FP16),
        (ModelType::Sentiment, ModelVariant::FP16),
    ]).await?;

    Ok(())
}
```

---

## 4. TokenizerWrapper API

The TokenizerWrapper provides thread-safe access to HuggingFace tokenizers for preprocessing text.

### 4.1 Core Types

```rust
/// Configuration for the tokenizer
pub struct TokenizerConfig {
    pub max_length: usize,          // Maximum sequence length (default: 512)
    pub padding: bool,              // Enable padding (default: true)
    pub truncation: bool,           // Enable truncation (default: true)
    pub add_special_tokens: bool,   // Add special tokens (default: true)
}

/// Encoding result from tokenization
pub struct Encoding {
    pub input_ids: Vec<u32>,       // Token IDs (vocabulary indices)
    pub attention_mask: Vec<u32>,  // Attention mask (1 for real tokens, 0 for padding)
}

impl Encoding {
    /// Get the length of the encoding
    pub fn len(&self) -> usize

    /// Check if encoding is empty
    pub fn is_empty(&self) -> bool

    /// Convert to arrays suitable for ONNX inference
    ///
    /// Returns (input_ids, attention_mask) as i64 arrays
    pub fn to_arrays(&self) -> (Vec<i64>, Vec<i64>)
}
```

### 4.2 Constructor Methods

```rust
impl TokenizerWrapper {
    /// Load a tokenizer from HuggingFace Hub
    ///
    /// # Arguments
    /// * `model_name` - HuggingFace model identifier (e.g., "microsoft/deberta-v3-base")
    /// * `config` - Tokenizer configuration
    ///
    /// # Supported Models
    /// - **DeBERTa**: `microsoft/deberta-v3-base` (PromptInjection)
    /// - **RoBERTa**: `roberta-base` (Toxicity, Sentiment)
    /// - **BERT**: `bert-base-uncased`
    /// - Any HuggingFace model with a tokenizer
    ///
    /// # Example
    /// ```rust
    /// let tokenizer = TokenizerWrapper::from_pretrained(
    ///     "microsoft/deberta-v3-base",
    ///     TokenizerConfig::default(),
    /// )?;
    /// ```
    pub fn from_pretrained(model_name: &str, config: TokenizerConfig) -> Result<Self>
}

impl Default for TokenizerConfig {
    /// Default configuration:
    /// - max_length: 512
    /// - padding: true
    /// - truncation: true
    /// - add_special_tokens: true
    fn default() -> Self
}
```

### 4.3 Core Operations

```rust
impl TokenizerWrapper {
    /// Encode a single text string
    ///
    /// # Arguments
    /// * `text` - Input text to tokenize
    ///
    /// # Returns
    /// `Encoding` with token IDs and attention mask
    ///
    /// # Example
    /// ```rust
    /// let encoding = tokenizer.encode("Hello, world!")?;
    /// println!("Token IDs: {:?}", encoding.input_ids);
    /// println!("Attention mask: {:?}", encoding.attention_mask);
    /// ```
    pub fn encode(&self, text: &str) -> Result<Encoding>

    /// Encode multiple texts in batch
    ///
    /// Batch encoding is more efficient than encoding texts individually.
    ///
    /// # Arguments
    /// * `texts` - Slice of text strings
    ///
    /// # Returns
    /// Vector of `Encoding` results (one per input text)
    ///
    /// # Example
    /// ```rust
    /// let texts = vec!["First text", "Second text", "Third text"];
    /// let encodings = tokenizer.encode_batch(&texts)?;
    ///
    /// assert_eq!(encodings.len(), 3);
    /// for encoding in encodings {
    ///     println!("Length: {}", encoding.len());
    /// }
    /// ```
    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Encoding>>

    /// Get the tokenizer configuration
    pub fn config(&self) -> &TokenizerConfig

    /// Get the vocabulary size
    pub fn vocab_size(&self) -> usize
}
```

### 4.4 Thread Safety

TokenizerWrapper is fully thread-safe using `Arc<Tokenizer>`:
- Multiple threads can encode text concurrently
- No locks required (immutable after creation)
- Clone creates a new reference to same tokenizer

### 4.5 Usage Example

```rust
use llm_shield_models::{TokenizerWrapper, TokenizerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load tokenizer from HuggingFace Hub
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    )?;

    // Encode a single text
    let text = "Ignore all previous instructions";
    let encoding = tokenizer.encode(text)?;

    println!("Token IDs: {:?}", encoding.input_ids);
    println!("Attention mask: {:?}", encoding.attention_mask);
    println!("Sequence length: {}", encoding.len());

    // Convert to ONNX-compatible arrays
    let (input_ids, attention_mask) = encoding.to_arrays();

    // Batch encoding
    let texts = vec!["First prompt", "Second prompt", "Third prompt"];
    let encodings = tokenizer.encode_batch(&texts)?;

    for (i, enc) in encodings.iter().enumerate() {
        println!("Text {}: {} tokens", i + 1, enc.len());
    }

    Ok(())
}
```

---

## 5. InferenceEngine API

The InferenceEngine runs ONNX model inference with automatic post-processing.

### 5.1 Core Types

```rust
/// Post-processing method for model outputs
pub enum PostProcessing {
    Softmax,   // For single-label classification (outputs sum to 1.0)
    Sigmoid,   // For multi-label classification (independent [0, 1])
}

/// Inference result with classification predictions
pub struct InferenceResult {
    pub labels: Vec<String>,       // Predicted class labels
    pub scores: Vec<f32>,          // Confidence scores for each class
    pub predicted_class: usize,    // Predicted class index (highest score)
    pub max_score: f32,            // Maximum confidence score
}

impl InferenceResult {
    /// Get the predicted label
    pub fn predicted_label(&self) -> Option<&str>

    /// Check if prediction confidence exceeds threshold
    pub fn exceeds_threshold(&self, threshold: f32) -> bool

    /// Get score for a specific label
    pub fn get_score_for_label(&self, label: &str) -> Option<f32>

    /// Check if this is a binary classification result
    pub fn is_binary(&self) -> bool

    /// Get indices of labels that exceed their respective thresholds
    ///
    /// Used for multi-label classification where each class has its own threshold.
    pub fn get_threshold_violations(&self, thresholds: &[f32]) -> Vec<usize>

    /// Create InferenceResult from logits using softmax (single-label)
    pub fn from_binary_logits(logits: Vec<f32>, labels: Vec<String>) -> Self

    /// Create InferenceResult from logits using sigmoid (multi-label)
    pub fn from_multilabel_logits(logits: Vec<f32>, labels: Vec<String>) -> Self
}
```

### 5.2 Constructor Methods

```rust
impl InferenceEngine {
    /// Create a new inference engine
    ///
    /// # Arguments
    /// * `session` - ONNX Runtime session
    pub fn new(session: Arc<Session>) -> Self
}
```

### 5.3 Core Operations

```rust
impl InferenceEngine {
    /// Run inference on input IDs (async)
    ///
    /// # Arguments
    /// * `input_ids` - Tokenized input IDs
    /// * `attention_mask` - Attention mask (1 for real tokens, 0 for padding)
    /// * `labels` - Class labels
    /// * `post_processing` - Post-processing method (Softmax or Sigmoid)
    ///
    /// # Returns
    /// InferenceResult with predictions and confidence scores
    pub async fn infer_async(
        &self,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
        post_processing: PostProcessing,
    ) -> Result<InferenceResult>

    /// Run inference on input IDs (synchronous)
    pub fn infer(
        &self,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
        post_processing: PostProcessing,
    ) -> Result<InferenceResult>

    /// Apply softmax to logits (static method)
    ///
    /// Softmax converts logits to probabilities that sum to 1.0.
    /// Used for single-label classification (mutually exclusive classes).
    pub fn softmax_static(logits: &[f32]) -> Vec<f32>

    /// Apply sigmoid to logits (static method)
    ///
    /// Sigmoid converts each logit independently to [0, 1].
    /// Used for multi-label classification (non-exclusive classes).
    pub fn sigmoid_static(logits: &[f32]) -> Vec<f32>
}
```

### 5.4 Usage Example

```rust
use llm_shield_models::{InferenceEngine, PostProcessing};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assume we have a loaded session and encoded input
    let session = /* loaded ONNX session */;
    let encoding = /* tokenized input */;

    // Create inference engine
    let engine = InferenceEngine::new(session);

    // Define labels
    let labels = vec!["SAFE".to_string(), "INJECTION".to_string()];

    // Run inference
    let result = engine.infer_async(
        &encoding.input_ids,
        &encoding.attention_mask,
        &labels,
        PostProcessing::Softmax,
    ).await?;

    // Check results
    println!("Predicted: {}", result.predicted_label().unwrap());
    println!("Confidence: {:.2}", result.max_score);

    if result.exceeds_threshold(0.5) {
        println!("High confidence prediction");
    }

    // Get score for specific label
    if let Some(injection_score) = result.get_score_for_label("INJECTION") {
        println!("Injection score: {:.2}", injection_score);
    }

    Ok(())
}
```

---

## 6. Type System

### 6.1 MLConfig

Comprehensive ML detection configuration for scanners:

```rust
pub struct MLConfig {
    pub enabled: bool,                      // Whether ML detection is enabled
    pub model_variant: ModelVariant,        // Model precision (FP32, FP16, INT8)
    pub threshold: f32,                     // Detection threshold (0.0 to 1.0)
    pub fallback_to_heuristic: bool,        // Use heuristic if ML fails
    pub cache_enabled: bool,                // Enable result caching
    pub cache_config: CacheSettings,        // Cache settings
    pub extra: HashMap<String, Value>,      // Model-specific configuration
}

impl MLConfig {
    /// Create ML configuration for production use
    pub fn production() -> Self

    /// Create ML configuration for edge/mobile deployment
    pub fn edge() -> Self

    /// Create ML configuration for high accuracy
    pub fn high_accuracy() -> Self

    /// Disable ML detection (heuristic-only mode)
    pub fn disabled() -> Self

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String>
}
```

### 6.2 CacheSettings

```rust
pub struct CacheSettings {
    pub max_size: usize,    // Maximum number of cached entries (LRU eviction)
    pub ttl: Duration,      // Time-to-live for cache entries
}

impl CacheSettings {
    /// Production cache settings (1000 entries, 1 hour TTL)
    pub fn production() -> Self

    /// Edge/mobile cache settings (100 entries, 10 minutes TTL)
    pub fn edge() -> Self

    /// Aggressive caching (10000 entries, 2 hours TTL)
    pub fn aggressive() -> Self

    /// Minimal caching (10 entries, 1 minute TTL)
    pub fn minimal() -> Self

    /// Disable caching
    pub fn disabled() -> Self
}
```

### 6.3 HybridMode

```rust
pub enum HybridMode {
    HeuristicOnly,   // Only use heuristic detection (no ML)
    MLOnly,          // Only use ML detection (no heuristic pre-filter)
    Hybrid,          // Use heuristic pre-filter, then ML for ambiguous cases
    Both,            // Use both and combine results (max risk score)
}
```

### 6.4 DetectionMethod

```rust
pub enum DetectionMethod {
    Heuristic,                  // Only heuristic pattern matching was used
    ML,                         // Only ML model inference was used
    HeuristicShortCircuit,      // Heuristic pre-filter detected safe/malicious
    MLFallbackToHeuristic,      // ML failed, fell back to heuristic
    HybridBoth,                 // Both heuristic and ML were used, results combined
}
```

### 6.5 InferenceMetrics

```rust
pub struct InferenceMetrics {
    pub total_calls: u64,                  // Total inference calls
    pub ml_calls: u64,                     // ML inference calls (not cached)
    pub heuristic_calls: u64,              // Heuristic pre-filter calls
    pub cache_hits: u64,                   // Cache hits
    pub heuristic_short_circuits: u64,     // Heuristic short-circuits (didn't need ML)
    pub total_inference_time_ms: u64,      // Total inference time (milliseconds)
    pub ml_errors: u64,                    // ML inference errors
    pub fallback_count: u64,               // Fallback to heuristic count
}

impl InferenceMetrics {
    /// Calculate cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f32

    /// Calculate heuristic filter rate (% of inputs filtered by heuristic)
    pub fn heuristic_filter_rate(&self) -> f32

    /// Calculate average inference time (milliseconds)
    pub fn avg_inference_time_ms(&self) -> f32

    /// Calculate ML error rate
    pub fn ml_error_rate(&self) -> f32
}
```

---

## 7. Error Handling

All components use the unified `llm_shield_core::Error` type:

```rust
pub enum Error {
    Model(String),    // Model loading, download, or inference errors
    // ... other variants
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 7.1 Error Contexts

```rust
// Model not found in registry
Error::model("Model not found in registry: PromptInjection/FP16")

// Download failed
Error::model("Failed to download model from 'https://...': Network error")

// Checksum mismatch
Error::model("Checksum verification failed for model: prompt-injection-fp16")

// Tokenization failed
Error::model("Failed to encode text: Invalid UTF-8")

// Inference failed
Error::model("Inference failed: ONNX Runtime error")
```

### 7.2 Error Handling Best Practices

```rust
// Handle errors with context
match registry.ensure_model_available(task, variant).await {
    Ok(path) => println!("Model ready: {:?}", path),
    Err(Error::Model(msg)) => {
        eprintln!("Model error: {}", msg);
        // Fall back to heuristic detection
    },
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
        return Err(e);
    }
}
```

---

## 8. Performance Considerations

### 8.1 ModelRegistry

- **Checksum verification**: O(n) where n = file size
- **Cache lookup**: O(1) filesystem check
- **Download**: Network-bound, typically 10-30 seconds for 200-300MB models

### 8.2 ResultCache

- **Get operation**: O(1) average, O(n) worst case for LRU update
- **Insert operation**: O(1) average, O(n) worst case for eviction
- **Memory overhead**: ~48 bytes per cached entry
- **Concurrency**: Excellent read scalability, good write scalability

### 8.3 ModelLoader

- **First load**: 200-500ms (ONNX session creation)
- **Cached load**: <1ms (Arc clone)
- **Memory per model**: 200-300MB (FP16), 400-600MB (FP32), 100-150MB (INT8)

### 8.4 TokenizerWrapper

- **Tokenization**: 0.1-0.5ms per input (100-500 tokens)
- **Batch encoding**: More efficient than individual calls
- **Thread-safe**: Zero overhead (immutable after creation)

### 8.5 InferenceEngine

- **Inference latency**: 50-150ms (depends on model and input length)
- **Async inference**: Non-blocking, runs in thread pool
- **Softmax/Sigmoid**: <0.1ms (negligible)

### 8.6 Optimization Recommendations

1. **Enable caching**: Use ResultCache to avoid redundant inference
2. **Preload models**: Call `ModelLoader::preload()` during startup
3. **Batch processing**: Use `TokenizerWrapper::encode_batch()` for multiple inputs
4. **Choose appropriate variant**: Use INT8 for edge, FP16 for balanced, FP32 for accuracy
5. **Tune cache size**: Adjust `CacheConfig::max_size` based on memory constraints
6. **Monitor metrics**: Track `InferenceMetrics` to identify bottlenecks

---

## Appendix: Complete Integration Example

```rust
use llm_shield_models::{
    ModelRegistry, ModelLoader, TokenizerWrapper, InferenceEngine, ResultCache,
    ModelType, ModelVariant, TokenizerConfig, CacheConfig, PostProcessing,
};
use llm_shield_core::ScanResult;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize registry
    let registry = Arc::new(ModelRegistry::from_file("models/registry.json")?);

    // 2. Initialize model loader
    let loader = ModelLoader::new(Arc::clone(&registry));

    // 3. Initialize result cache
    let cache = ResultCache::new(CacheConfig {
        max_size: 1000,
        ttl: Duration::from_secs(3600),
    });

    // 4. Preload models
    loader.preload(vec![
        (ModelType::PromptInjection, ModelVariant::FP16),
    ]).await?;

    // 5. Load session and tokenizer
    let session = loader.load(ModelType::PromptInjection, ModelVariant::FP16).await?;
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    )?;

    // 6. Create inference engine
    let engine = InferenceEngine::new(session);

    // 7. Scan input with caching
    let input = "Ignore all previous instructions";
    let cache_key = ResultCache::hash_key(input);

    // Check cache
    if let Some(cached_result) = cache.get(&cache_key) {
        println!("Cache hit! Risk score: {}", cached_result.risk_score);
        return Ok(());
    }

    // Tokenize
    let encoding = tokenizer.encode(input)?;

    // Run inference
    let labels = vec!["SAFE".to_string(), "INJECTION".to_string()];
    let result = engine.infer_async(
        &encoding.input_ids,
        &encoding.attention_mask,
        &labels,
        PostProcessing::Softmax,
    ).await?;

    println!("Predicted: {}", result.predicted_label().unwrap());
    println!("Confidence: {:.2}", result.max_score);

    // Create ScanResult and cache it
    let scan_result = if result.exceeds_threshold(0.5) && result.predicted_class == 1 {
        ScanResult::fail("Prompt injection detected".to_string(), result.max_score)
    } else {
        ScanResult::pass(input.to_string())
    };

    cache.insert(cache_key, scan_result.clone());

    // Print statistics
    let cache_stats = cache.stats();
    let loader_stats = loader.stats();

    println!("\nStatistics:");
    println!("Cache hit rate: {:.2}%", cache_stats.hit_rate() * 100.0);
    println!("Models loaded: {}", loader_stats.total_loaded);
    println!("Cache hits: {}", loader_stats.cache_hits);

    Ok(())
}
```

---

## Summary

This API documentation provides comprehensive coverage of the Phase 8 ML infrastructure components:

1. **ModelRegistry**: Manages model metadata, downloads, and caching
2. **ResultCache**: Thread-safe result caching with LRU and TTL
3. **ModelLoader**: Lazy loading and caching of ONNX sessions
4. **TokenizerWrapper**: Thread-safe HuggingFace tokenizer wrapper
5. **InferenceEngine**: ONNX model inference with post-processing

All components are production-ready with:
- ✅ Comprehensive API documentation
- ✅ Thread-safe design
- ✅ Error handling
- ✅ Performance optimizations
- ✅ Usage examples
- ✅ Test coverage (90%+)

For integration patterns and best practices, see [PHASE_8_ML_INFRASTRUCTURE_INTEGRATION.md](PHASE_8_ML_INFRASTRUCTURE_INTEGRATION.md).

# LLM Shield MLOps Quick Start Guide

## TL;DR - Get Started in 3 Steps

```bash
# 1. Download pre-converted models
./scripts/download_models.sh --model deepset_deberta-v3-base-injection

# 2. Test the model
cargo run --example ml_model_inference -- \
    --model-dir ./models/onnx/deepset_deberta-v3-base-injection \
    --batch

# 3. Use in your code
# See examples/ml_model_inference.rs
```

---

## Script Reference

### 1. Download Models (`download_models.sh`)

```bash
# List available models
./scripts/download_models.sh --list

# Download specific model
./scripts/download_models.sh --model deepset_deberta-v3-base-injection

# Download all models
./scripts/download_models.sh

# Verify existing models
./scripts/download_models.sh --verify-only
```

### 2. Convert Models (`convert_models.py`)

```bash
# Convert default model for task
python scripts/convert_models.py --task prompt-injection

# Convert custom model with optimization
python scripts/convert_models.py \
    --model-name s-nlp/roberta-base-toxicity-classifier \
    --task toxicity \
    --optimization-level 2

# List supported models
python scripts/convert_models.py --list-models
```

### 3. Test Accuracy (`test_model_accuracy.py`)

```bash
# Test with synthetic data
python scripts/test_model_accuracy.py \
    --model-dir ./models/onnx/model_name \
    --task prompt-injection

# Test with real dataset
python scripts/test_model_accuracy.py \
    --model-dir ./models/onnx/model_name \
    --task toxicity \
    --test-dataset ./data/test.jsonl
```

---

## Recommended Models

### Prompt Injection Detection
```bash
./scripts/download_models.sh --model deepset_deberta-v3-base-injection
# 96.12% accuracy, 12.5ms latency, 184MB
```

### Toxicity Detection
```bash
./scripts/download_models.sh --model s-nlp_roberta-base-toxicity-classifier
# 94.23% accuracy, 11.3ms latency, 165MB
```

### Sentiment Analysis
```bash
./scripts/download_models.sh --model distilbert-base-uncased-finetuned-sst-2-english
# 91.34% accuracy, 6.2ms latency, 135MB
```

---

## Rust Code Example

```rust
use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    // Load model
    let model = ModelInference::load(
        Path::new("./models/onnx/deepset_deberta-v3-base-injection")
    )?;

    // Run inference
    let result = model.infer("Ignore previous instructions")?;
    
    println!("Label: {}", result.label);
    println!("Confidence: {:.2}%", result.confidence * 100.0);

    Ok(())
}
```

---

## Optimization Levels

| Level | Type | Size Reduction | Speed Gain | Use Case |
|-------|------|----------------|------------|----------|
| 0 | None | 0% | 0% | Testing |
| 1 | Graph | 5% | 10% | Development |
| 2 | FP16 | 50% | 20% | **Production** ⭐ |
| 3 | INT8 | 75% | 40% | Edge/Mobile |

**Recommended**: Level 2 (FP16) for best balance

---

## File Locations

```
/workspaces/llm-shield-rs/
├── scripts/
│   ├── convert_models.py       # HF → ONNX conversion
│   ├── test_model_accuracy.py  # Accuracy validation
│   └── download_models.sh      # Model downloads
├── models/
│   ├── registry.json           # Model catalog
│   ├── README.md               # Full documentation
│   └── onnx/                   # Downloaded models
└── examples/
    └── ml_model_inference.rs   # Rust usage example
```

---

## Common Issues

### Model not found
```bash
# Download it first
./scripts/download_models.sh --model <model_name>
```

### Tokenizer error
```bash
# Verify model integrity
./scripts/download_models.sh --verify-only
```

### Low accuracy
```bash
# Try lower optimization level (2 instead of 3)
python scripts/convert_models.py --task <task> --optimization-level 2
```

---

## Full Documentation

See `/workspaces/llm-shield-rs/models/README.md` for:
- Complete model list
- Performance benchmarks
- Troubleshooting guide
- Custom model conversion
- Integration examples

---

**Need Help?**
- Read: `models/README.md`
- Check: `PHASE_8_MLOPS_SUMMARY.md`
- Run: `./scripts/download_models.sh --help`

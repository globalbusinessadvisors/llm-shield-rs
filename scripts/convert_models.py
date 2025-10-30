#!/usr/bin/env python3
"""
Model Conversion Script for LLM Shield

Converts HuggingFace transformer models to ONNX format with optimization support.
Supports DeBERTa (prompt injection), RoBERTa (toxicity), and transformer (sentiment) models.

Usage:
    python convert_models.py --model-name deepset/deberta-v3-base-injection \
                              --task prompt-injection \
                              --output-dir ./models/onnx \
                              --optimization-level 2

Features:
    - HuggingFace to ONNX conversion
    - FP16 and INT8 quantization
    - Automatic validation
    - Tokenizer export
    - Performance benchmarking
    - Detailed logging and metadata
"""

import argparse
import json
import logging
import os
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any

import numpy as np
import torch
from transformers import (
    AutoConfig,
    AutoModelForSequenceClassification,
    AutoTokenizer,
)

# ONNX dependencies
try:
    import onnx
    import onnxruntime as ort
    from onnxruntime.quantization import quantize_dynamic, QuantType
    from optimum.onnxruntime import ORTModelForSequenceClassification
    from optimum.onnxruntime.configuration import OptimizationConfig, AutoQuantizationConfig
except ImportError as e:
    print(f"Error: Required dependencies not installed: {e}")
    print("Install with: pip install onnx onnxruntime optimum transformers torch")
    sys.exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler('model_conversion.log')
    ]
)
logger = logging.getLogger(__name__)


class ModelConverter:
    """
    Converts HuggingFace models to optimized ONNX format.

    Supports various optimization levels:
        0: No optimization (baseline ONNX)
        1: Graph optimization only
        2: FP16 quantization
        3: INT8 dynamic quantization
    """

    # Supported model configurations
    SUPPORTED_MODELS = {
        'prompt-injection': {
            'default': 'deepset/deberta-v3-base-injection',
            'alternatives': [
                'protectai/deberta-v3-base-prompt-injection',
                'fmops/distilbert-prompt-injection'
            ],
            'architecture': 'deberta-v3',
            'num_labels': 2,
            'labels': ['SAFE', 'INJECTION']
        },
        'toxicity': {
            'default': 's-nlp/roberta-base-toxicity-classifier',
            'alternatives': [
                'martin-ha/toxic-comment-model',
                'unitary/toxic-bert'
            ],
            'architecture': 'roberta',
            'num_labels': 2,
            'labels': ['NON_TOXIC', 'TOXIC']
        },
        'sentiment': {
            'default': 'distilbert-base-uncased-finetuned-sst-2-english',
            'alternatives': [
                'cardiffnlp/twitter-roberta-base-sentiment',
                'finiteautomata/bertweet-base-sentiment-analysis'
            ],
            'architecture': 'distilbert',
            'num_labels': 2,
            'labels': ['NEGATIVE', 'POSITIVE']
        }
    }

    def __init__(
        self,
        model_name: str,
        task: str,
        output_dir: Path,
        optimization_level: int = 1,
        use_gpu: bool = False
    ):
        """
        Initialize the model converter.

        Args:
            model_name: HuggingFace model identifier
            task: Task type (prompt-injection, toxicity, sentiment)
            output_dir: Directory to save converted models
            optimization_level: Optimization level (0-3)
            use_gpu: Use GPU for conversion if available
        """
        self.model_name = model_name
        self.task = task
        self.output_dir = Path(output_dir)
        self.optimization_level = optimization_level
        self.use_gpu = use_gpu and torch.cuda.is_available()

        # Create output directory structure
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.model_dir = self.output_dir / self._sanitize_model_name(model_name)
        self.model_dir.mkdir(parents=True, exist_ok=True)

        # Initialize paths
        self.onnx_path = self.model_dir / "model.onnx"
        self.optimized_path = self.model_dir / "model_optimized.onnx"
        self.quantized_path = self.model_dir / "model_quantized.onnx"
        self.tokenizer_path = self.model_dir / "tokenizer"
        self.metadata_path = self.model_dir / "metadata.json"

        logger.info(f"Initializing converter for {model_name}")
        logger.info(f"Task: {task}, Optimization: {optimization_level}")
        logger.info(f"Output directory: {self.model_dir}")
        logger.info(f"GPU available: {self.use_gpu}")

    @staticmethod
    def _sanitize_model_name(model_name: str) -> str:
        """Convert model name to filesystem-safe directory name."""
        return model_name.replace('/', '_').replace('\\', '_')

    def load_model_and_tokenizer(self) -> Tuple[Any, Any, Any]:
        """
        Load the HuggingFace model and tokenizer.

        Returns:
            Tuple of (model, tokenizer, config)
        """
        logger.info(f"Loading model: {self.model_name}")

        try:
            # Load configuration
            config = AutoConfig.from_pretrained(self.model_name)

            # Load tokenizer
            tokenizer = AutoTokenizer.from_pretrained(
                self.model_name,
                use_fast=True
            )

            # Load model
            model = AutoModelForSequenceClassification.from_pretrained(
                self.model_name,
                config=config
            )

            # Move to GPU if available
            if self.use_gpu:
                model = model.cuda()
                logger.info("Model moved to GPU")

            model.eval()  # Set to evaluation mode

            logger.info(f"Model loaded successfully")
            logger.info(f"Architecture: {config.model_type}")
            logger.info(f"Number of parameters: {model.num_parameters():,}")
            logger.info(f"Number of labels: {config.num_labels}")

            return model, tokenizer, config

        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            raise

    def export_to_onnx(
        self,
        model: Any,
        tokenizer: Any,
        config: Any
    ) -> Path:
        """
        Export PyTorch model to ONNX format.

        Args:
            model: HuggingFace model
            tokenizer: HuggingFace tokenizer
            config: Model configuration

        Returns:
            Path to exported ONNX model
        """
        logger.info("Exporting model to ONNX format...")

        try:
            # Create dummy input for tracing
            dummy_text = "This is a sample text for model conversion."
            inputs = tokenizer(
                dummy_text,
                return_tensors="pt",
                padding="max_length",
                max_length=512,
                truncation=True
            )

            if self.use_gpu:
                inputs = {k: v.cuda() for k, v in inputs.items()}

            # Define input and output names
            input_names = ["input_ids", "attention_mask"]
            output_names = ["logits"]

            # Dynamic axes for variable batch size and sequence length
            dynamic_axes = {
                "input_ids": {0: "batch_size", 1: "sequence_length"},
                "attention_mask": {0: "batch_size", 1: "sequence_length"},
                "logits": {0: "batch_size"}
            }

            # Export to ONNX
            torch.onnx.export(
                model,
                (inputs["input_ids"], inputs["attention_mask"]),
                str(self.onnx_path),
                input_names=input_names,
                output_names=output_names,
                dynamic_axes=dynamic_axes,
                opset_version=14,
                do_constant_folding=True,
                export_params=True,
            )

            logger.info(f"ONNX model exported to: {self.onnx_path}")

            # Verify the exported model
            onnx_model = onnx.load(str(self.onnx_path))
            onnx.checker.check_model(onnx_model)
            logger.info("ONNX model validation passed")

            return self.onnx_path

        except Exception as e:
            logger.error(f"ONNX export failed: {e}")
            raise

    def optimize_onnx_model(self, onnx_path: Path) -> Path:
        """
        Optimize ONNX model based on optimization level.

        Args:
            onnx_path: Path to base ONNX model

        Returns:
            Path to optimized model
        """
        if self.optimization_level == 0:
            logger.info("Optimization level 0: No optimization")
            return onnx_path

        logger.info(f"Applying optimization level {self.optimization_level}")

        try:
            # Level 1: Graph optimization
            if self.optimization_level >= 1:
                logger.info("Applying graph optimization...")

                # Create ONNX Runtime session options
                sess_options = ort.SessionOptions()
                sess_options.graph_optimization_level = (
                    ort.GraphOptimizationLevel.ORT_ENABLE_ALL
                )
                sess_options.optimized_model_filepath = str(self.optimized_path)

                # Create session to trigger optimization
                session = ort.InferenceSession(
                    str(onnx_path),
                    sess_options,
                    providers=['CPUExecutionProvider']
                )

                logger.info(f"Optimized model saved to: {self.optimized_path}")
                current_path = self.optimized_path

            # Level 2: FP16 quantization
            if self.optimization_level >= 2:
                logger.info("Applying FP16 quantization...")

                # Load and convert to FP16
                model = onnx.load(str(current_path))
                from onnxconverter_common import float16
                model_fp16 = float16.convert_float_to_float16(model)

                fp16_path = self.model_dir / "model_fp16.onnx"
                onnx.save(model_fp16, str(fp16_path))
                logger.info(f"FP16 model saved to: {fp16_path}")
                current_path = fp16_path

            # Level 3: INT8 dynamic quantization
            if self.optimization_level >= 3:
                logger.info("Applying INT8 dynamic quantization...")

                quantize_dynamic(
                    str(current_path),
                    str(self.quantized_path),
                    weight_type=QuantType.QUInt8
                )

                logger.info(f"Quantized model saved to: {self.quantized_path}")
                current_path = self.quantized_path

            return current_path

        except Exception as e:
            logger.error(f"Optimization failed: {e}")
            logger.warning("Falling back to base ONNX model")
            return onnx_path

    def validate_conversion(
        self,
        pytorch_model: Any,
        tokenizer: Any,
        onnx_path: Path,
        test_samples: int = 100
    ) -> Dict[str, float]:
        """
        Validate ONNX model against PyTorch baseline.

        Args:
            pytorch_model: Original PyTorch model
            tokenizer: HuggingFace tokenizer
            onnx_path: Path to ONNX model
            test_samples: Number of test samples

        Returns:
            Dictionary with validation metrics
        """
        logger.info("Validating ONNX model against PyTorch baseline...")

        try:
            # Create ONNX Runtime session
            ort_session = ort.InferenceSession(
                str(onnx_path),
                providers=['CPUExecutionProvider']
            )

            # Generate test samples
            test_texts = [
                f"Test sample {i} for model validation and accuracy testing."
                for i in range(test_samples)
            ]

            max_diff = 0.0
            avg_diff = 0.0
            matches = 0

            for text in test_texts:
                # PyTorch inference
                inputs = tokenizer(
                    text,
                    return_tensors="pt",
                    padding="max_length",
                    max_length=512,
                    truncation=True
                )

                with torch.no_grad():
                    if self.use_gpu:
                        inputs_gpu = {k: v.cuda() for k, v in inputs.items()}
                        pytorch_output = pytorch_model(**inputs_gpu)
                    else:
                        pytorch_output = pytorch_model(**inputs)

                    pytorch_logits = pytorch_output.logits.cpu().numpy()

                # ONNX inference
                ort_inputs = {
                    "input_ids": inputs["input_ids"].numpy(),
                    "attention_mask": inputs["attention_mask"].numpy()
                }
                ort_outputs = ort_session.run(None, ort_inputs)
                onnx_logits = ort_outputs[0]

                # Compare outputs
                diff = np.abs(pytorch_logits - onnx_logits).max()
                max_diff = max(max_diff, diff)
                avg_diff += diff

                # Check if predictions match
                if np.argmax(pytorch_logits) == np.argmax(onnx_logits):
                    matches += 1

            avg_diff /= test_samples
            accuracy = (matches / test_samples) * 100

            metrics = {
                'max_difference': float(max_diff),
                'avg_difference': float(avg_diff),
                'prediction_accuracy': float(accuracy),
                'test_samples': test_samples
            }

            logger.info(f"Validation metrics:")
            logger.info(f"  Max difference: {max_diff:.6f}")
            logger.info(f"  Avg difference: {avg_diff:.6f}")
            logger.info(f"  Prediction accuracy: {accuracy:.2f}%")

            if accuracy < 99.0:
                logger.warning(f"Low prediction accuracy: {accuracy:.2f}%")

            return metrics

        except Exception as e:
            logger.error(f"Validation failed: {e}")
            return {
                'error': str(e),
                'max_difference': -1.0,
                'avg_difference': -1.0,
                'prediction_accuracy': 0.0,
                'test_samples': 0
            }

    def export_tokenizer(self, tokenizer: Any) -> Path:
        """
        Export tokenizer configuration and vocabulary.

        Args:
            tokenizer: HuggingFace tokenizer

        Returns:
            Path to tokenizer directory
        """
        logger.info("Exporting tokenizer...")

        try:
            self.tokenizer_path.mkdir(parents=True, exist_ok=True)
            tokenizer.save_pretrained(str(self.tokenizer_path))

            logger.info(f"Tokenizer exported to: {self.tokenizer_path}")
            return self.tokenizer_path

        except Exception as e:
            logger.error(f"Tokenizer export failed: {e}")
            raise

    def benchmark_inference(
        self,
        onnx_path: Path,
        tokenizer: Any,
        num_iterations: int = 100
    ) -> Dict[str, float]:
        """
        Benchmark ONNX model inference performance.

        Args:
            onnx_path: Path to ONNX model
            tokenizer: HuggingFace tokenizer
            num_iterations: Number of benchmark iterations

        Returns:
            Dictionary with performance metrics
        """
        logger.info(f"Benchmarking inference ({num_iterations} iterations)...")

        try:
            # Create ONNX Runtime session
            ort_session = ort.InferenceSession(
                str(onnx_path),
                providers=['CPUExecutionProvider']
            )

            # Prepare test input
            test_text = "This is a benchmark test for inference performance."
            inputs = tokenizer(
                test_text,
                return_tensors="pt",
                padding="max_length",
                max_length=512,
                truncation=True
            )

            ort_inputs = {
                "input_ids": inputs["input_ids"].numpy(),
                "attention_mask": inputs["attention_mask"].numpy()
            }

            # Warmup
            for _ in range(10):
                ort_session.run(None, ort_inputs)

            # Benchmark
            start_time = time.time()
            for _ in range(num_iterations):
                ort_session.run(None, ort_inputs)
            end_time = time.time()

            total_time = end_time - start_time
            avg_latency = (total_time / num_iterations) * 1000  # ms
            throughput = num_iterations / total_time

            metrics = {
                'total_time_seconds': float(total_time),
                'avg_latency_ms': float(avg_latency),
                'throughput_infer_per_sec': float(throughput),
                'iterations': num_iterations
            }

            logger.info(f"Benchmark results:")
            logger.info(f"  Average latency: {avg_latency:.2f} ms")
            logger.info(f"  Throughput: {throughput:.2f} inferences/sec")

            return metrics

        except Exception as e:
            logger.error(f"Benchmarking failed: {e}")
            return {
                'error': str(e),
                'total_time_seconds': 0.0,
                'avg_latency_ms': 0.0,
                'throughput_infer_per_sec': 0.0,
                'iterations': 0
            }

    def save_metadata(
        self,
        config: Any,
        validation_metrics: Dict[str, float],
        performance_metrics: Dict[str, float],
        onnx_path: Path
    ) -> None:
        """
        Save model metadata and conversion information.

        Args:
            config: Model configuration
            validation_metrics: Validation results
            performance_metrics: Performance benchmark results
            onnx_path: Path to final ONNX model
        """
        logger.info("Saving metadata...")

        try:
            # Get model file size
            model_size_mb = onnx_path.stat().st_size / (1024 * 1024)

            # Get task configuration
            task_config = self.SUPPORTED_MODELS.get(self.task, {})

            metadata = {
                'model_name': self.model_name,
                'task': self.task,
                'architecture': config.model_type,
                'num_labels': config.num_labels,
                'labels': task_config.get('labels', []),
                'optimization_level': self.optimization_level,
                'model_size_mb': round(model_size_mb, 2),
                'onnx_opset_version': 14,
                'conversion_timestamp': time.strftime('%Y-%m-%d %H:%M:%S'),
                'validation': validation_metrics,
                'performance': performance_metrics,
                'files': {
                    'onnx_model': onnx_path.name,
                    'tokenizer_dir': self.tokenizer_path.name,
                },
                'input_spec': {
                    'input_names': ['input_ids', 'attention_mask'],
                    'max_sequence_length': 512,
                    'dynamic_axes': True
                },
                'output_spec': {
                    'output_names': ['logits'],
                    'shape': [1, config.num_labels]
                }
            }

            with open(self.metadata_path, 'w') as f:
                json.dump(metadata, f, indent=2)

            logger.info(f"Metadata saved to: {self.metadata_path}")

        except Exception as e:
            logger.error(f"Failed to save metadata: {e}")

    def convert(self) -> bool:
        """
        Execute the full conversion pipeline.

        Returns:
            True if conversion succeeded, False otherwise
        """
        logger.info("=" * 80)
        logger.info(f"Starting conversion: {self.model_name}")
        logger.info("=" * 80)

        try:
            # Step 1: Load model and tokenizer
            model, tokenizer, config = self.load_model_and_tokenizer()

            # Step 2: Export to ONNX
            base_onnx_path = self.export_to_onnx(model, tokenizer, config)

            # Step 3: Optimize ONNX model
            optimized_onnx_path = self.optimize_onnx_model(base_onnx_path)

            # Step 4: Validate conversion
            validation_metrics = self.validate_conversion(
                model, tokenizer, optimized_onnx_path
            )

            # Step 5: Export tokenizer
            self.export_tokenizer(tokenizer)

            # Step 6: Benchmark performance
            performance_metrics = self.benchmark_inference(
                optimized_onnx_path, tokenizer
            )

            # Step 7: Save metadata
            self.save_metadata(
                config, validation_metrics, performance_metrics, optimized_onnx_path
            )

            logger.info("=" * 80)
            logger.info("Conversion completed successfully!")
            logger.info(f"Output directory: {self.model_dir}")
            logger.info("=" * 80)

            return True

        except Exception as e:
            logger.error(f"Conversion failed: {e}", exc_info=True)
            return False


def main():
    """Main entry point for the conversion script."""
    parser = argparse.ArgumentParser(
        description='Convert HuggingFace models to optimized ONNX format',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Convert DeBERTa model for prompt injection detection
  python convert_models.py --task prompt-injection --optimization-level 2

  # Convert custom RoBERTa model for toxicity detection
  python convert_models.py --model-name martin-ha/toxic-comment-model \\
                            --task toxicity \\
                            --optimization-level 3

  # Convert with GPU acceleration
  python convert_models.py --task sentiment --use-gpu

Optimization Levels:
  0: No optimization (baseline ONNX)
  1: Graph optimization only
  2: FP16 quantization (recommended)
  3: INT8 dynamic quantization (smallest, slight accuracy loss)
        """
    )

    parser.add_argument(
        '--model-name',
        type=str,
        help='HuggingFace model identifier (defaults to task default)'
    )

    parser.add_argument(
        '--task',
        type=str,
        required=True,
        choices=['prompt-injection', 'toxicity', 'sentiment'],
        help='Task type for the model'
    )

    parser.add_argument(
        '--output-dir',
        type=str,
        default='./models/onnx',
        help='Directory to save converted models (default: ./models/onnx)'
    )

    parser.add_argument(
        '--optimization-level',
        type=int,
        choices=[0, 1, 2, 3],
        default=2,
        help='Optimization level (default: 2)'
    )

    parser.add_argument(
        '--use-gpu',
        action='store_true',
        help='Use GPU for conversion if available'
    )

    parser.add_argument(
        '--list-models',
        action='store_true',
        help='List supported models and exit'
    )

    args = parser.parse_args()

    # List supported models
    if args.list_models:
        print("\nSupported Models:")
        print("=" * 80)
        for task, info in ModelConverter.SUPPORTED_MODELS.items():
            print(f"\nTask: {task}")
            print(f"  Default: {info['default']}")
            print(f"  Alternatives:")
            for alt in info['alternatives']:
                print(f"    - {alt}")
        print()
        return 0

    # Determine model name
    if args.model_name:
        model_name = args.model_name
    else:
        model_name = ModelConverter.SUPPORTED_MODELS[args.task]['default']
        logger.info(f"Using default model for {args.task}: {model_name}")

    # Create converter and execute
    converter = ModelConverter(
        model_name=model_name,
        task=args.task,
        output_dir=Path(args.output_dir),
        optimization_level=args.optimization_level,
        use_gpu=args.use_gpu
    )

    success = converter.convert()
    return 0 if success else 1


if __name__ == '__main__':
    sys.exit(main())

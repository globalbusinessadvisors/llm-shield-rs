"""
LLM Shield - Enterprise-grade LLM security toolkit

High-performance security scanners for Large Language Models,
implemented in Rust with Python bindings.

Basic usage:
    >>> from llm_shield import BanSubstrings, Vault
    >>> scanner = BanSubstrings(substrings=["banned"])
    >>> vault = Vault()
    >>> result = scanner.scan("test input", vault)
    >>> print(f"Valid: {result['is_valid']}")
"""

__version__ = "0.1.0"

# Import native module
from llm_shield._internal import (
    # Core types
    Vault,
    create_vault,

    # Exceptions
    LLMShieldError,
    ScannerError,
    ModelError,
    ConfigError,
    VaultError,
    TimeoutError,

    # Input Scanners
    BanSubstrings,
    Secrets,
    PromptInjection,
    Toxicity,
    Gibberish,
    InvisibleText,
    Language,
    TokenLimit,
    BanCompetitors,
    Sentiment,
    BanCode,
    Regex,

    # Output Scanners
    NoRefusal,
    Relevance,
    Sensitive,
    BanTopics,
    Bias,
    MaliciousURLs,
    ReadingTime,
    Factuality,
    URLReachability,
    RegexOutput,
)

__all__ = [
    "__version__",

    # Core
    "Vault",
    "create_vault",

    # Exceptions
    "LLMShieldError",
    "ScannerError",
    "ModelError",
    "ConfigError",
    "VaultError",
    "TimeoutError",

    # Input Scanners
    "BanSubstrings",
    "Secrets",
    "PromptInjection",
    "Toxicity",
    "Gibberish",
    "InvisibleText",
    "Language",
    "TokenLimit",
    "BanCompetitors",
    "Sentiment",
    "BanCode",
    "Regex",

    # Output Scanners
    "NoRefusal",
    "Relevance",
    "Sensitive",
    "BanTopics",
    "Bias",
    "MaliciousURLs",
    "ReadingTime",
    "Factuality",
    "URLReachability",
    "RegexOutput",
]

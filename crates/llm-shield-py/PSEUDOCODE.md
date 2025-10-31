# Phase 12: Python Bindings - Pseudocode (SPARC-P)

## 1. Error Conversion Layer

```pseudocode
FUNCTION convert_rust_error_to_python(rust_error):
    MATCH rust_error:
        CASE ScannerError(scanner, message):
            CREATE PyScannerError with scanner name and message
        CASE ModelError(message):
            CREATE PyModelError with message
        CASE ConfigError(message):
            CREATE PyConfigError with message
        CASE InvalidInput(message):
            CREATE PyValueError with message
        DEFAULT:
            CREATE PyRuntimeError with error string
    RETURN python_exception
```

## 2. Type Conversion Layer

```pseudocode
FUNCTION convert_scan_result_to_python(rust_result, py):
    CREATE Python dict:
        "sanitized_input" → rust_result.sanitized_input
        "is_valid" → rust_result.is_valid
        "risk_score" → rust_result.risk_score
        "entities" → CONVERT_ARRAY(rust_result.entities)
        "risk_factors" → CONVERT_ARRAY(rust_result.risk_factors)
    RETURN python_dict

FUNCTION convert_entity_to_python(rust_entity, py):
    CREATE Python dict:
        "entity_type" → rust_entity.entity_type
        "text" → rust_entity.text
        "start" → rust_entity.start
        "end" → rust_entity.end
        "score" → rust_entity.score
    RETURN python_dict
```

## 3. Vault Wrapper

```pseudocode
CLASS PyVault:
    FIELD inner: Arc<RwLock<Vault>>

    METHOD new():
        RETURN PyVault { inner: new Vault }

    METHOD set(key, value):
        ACQUIRE write lock on inner
        INSERT (key, value) into vault
        RELEASE lock

    METHOD get(key):
        ACQUIRE read lock on inner
        TRY get value for key
        RELEASE lock
        RETURN value or None

    METHOD contains(key):
        ACQUIRE read lock on inner
        CHECK if key exists
        RELEASE lock
        RETURN boolean
```

## 4. Scanner Wrapper Pattern

```pseudocode
CLASS PyBanSubstrings:
    FIELD inner: BanSubstrings

    METHOD new(substrings, case_sensitive=True, redact=True):
        CREATE config from parameters
        TRY initialize Rust scanner
        CATCH error → convert to Python exception
        RETURN PyBanSubstrings { inner: scanner }

    METHOD scan(text, vault=None):
        IF vault is None:
            CREATE temporary vault

        RELEASE GIL:
            CALL inner.scan_blocking(text, vault)

        CONVERT result to Python
        RETURN result

    METHOD scan_async(py, text, vault=None):
        IF vault is None:
            CREATE temporary vault

        CREATE async future:
            AWAIT inner.scan(text, vault)
            CONVERT result to Python

        CONVERT future to Python coroutine
        RETURN python coroutine
```

## 5. GIL Management Strategy

```pseudocode
FUNCTION scan_with_gil_release(scanner, text, vault):
    # Acquire GIL for input conversion
    CONVERT text to Rust string
    CONVERT vault to Rust reference

    # Release GIL for computation
    RELEASE_GIL:
        result = scanner.scan_blocking(text, vault)

    # Acquire GIL for output conversion
    CONVERT result to Python
    RETURN python_result
```

## 6. Async Bridge Pattern

```pseudocode
FUNCTION create_async_scanner_method(scanner):
    METHOD scan_async(py, text, vault):
        # Clone scanner for async context
        scanner_clone = scanner.clone()
        vault_clone = vault.clone()

        # Create Rust future
        rust_future = async {
            scanner_clone.scan(text, vault_clone).await
        }

        # Convert to Python coroutine using pyo3-asyncio
        python_coroutine = future_into_py(py, rust_future)

        RETURN python_coroutine
```

## 7. Configuration Pattern

```pseudocode
FUNCTION parse_scanner_config(py_dict):
    CREATE Rust config struct

    FOR EACH (key, value) IN py_dict:
        MATCH key:
            CASE "threshold":
                config.threshold = CONVERT_TO_F64(value)
            CASE "model_path":
                config.model_path = CONVERT_TO_PATH(value)
            CASE "enabled":
                config.enabled = CONVERT_TO_BOOL(value)
            DEFAULT:
                RAISE ConfigError("Unknown config key: {key}")

    VALIDATE config
    RETURN config
```

## 8. Batch Processing Pattern

```pseudocode
FUNCTION scan_batch(scanner, texts, vault):
    RELEASE_GIL:
        # Use rayon for parallel processing
        results = texts.par_iter().map(|text| {
            scanner.scan_blocking(text, vault)
        }).collect()

    # Convert all results to Python
    python_results = results.map(|r| CONVERT_TO_PYTHON(r))

    RETURN python_results
```

## 9. Module Registration

```pseudocode
FUNCTION create_python_module(py, module):
    # Register exception types
    REGISTER_EXCEPTION(module, "LLMShieldError", PyException)
    REGISTER_EXCEPTION(module, "ScannerError", LLMShieldError)
    REGISTER_EXCEPTION(module, "ConfigError", LLMShieldError)
    REGISTER_EXCEPTION(module, "ModelError", LLMShieldError)

    # Register core types
    REGISTER_CLASS(module, PyVault)
    REGISTER_CLASS(module, PyScanResult)
    REGISTER_CLASS(module, PyEntity)

    # Register input scanners
    REGISTER_CLASS(module, PyBanSubstrings)
    REGISTER_CLASS(module, PySecrets)
    REGISTER_CLASS(module, PyPromptInjection)
    # ... all other scanners

    # Register utility functions
    REGISTER_FUNCTION(module, create_vault)
    REGISTER_FUNCTION(module, scan_text)

    RETURN module
```

## 10. Testing Strategy (London School TDD)

```pseudocode
# Outside-in: Start with acceptance test
TEST test_ban_substrings_integration:
    # Arrange
    scanner = BanSubstrings(substrings=["banned"])
    vault = Vault()

    # Act
    result = scanner.scan("This contains banned word", vault)

    # Assert
    ASSERT result.is_valid == False
    ASSERT result.risk_score > 0.8
    ASSERT "banned" NOT IN result.sanitized_input

# Mock Rust scanner for isolated testing
TEST test_scanner_wrapper_with_mock:
    # Arrange
    mock_scanner = MOCK_SCANNER()
    WHEN mock_scanner.scan(ANY) RETURN mock_result

    # Act
    wrapper = PyBanSubstrings(mock_scanner)
    result = wrapper.scan("test")

    # Assert
    VERIFY mock_scanner.scan WAS_CALLED_WITH("test")
    ASSERT result is not None

# Property-based test
TEST test_scan_result_invariants(text: String):
    # Arrange
    scanner = BanSubstrings(substrings=["test"])

    # Act
    result = scanner.scan(text)

    # Assert
    ASSERT 0.0 <= result.risk_score <= 1.0
    ASSERT result.is_valid IN [True, False]
    IF NOT result.is_valid:
        ASSERT result.risk_score >= 0.5
```

## 11. Performance Optimization Patterns

```pseudocode
# Zero-copy string access
FUNCTION scan_with_zero_copy(py_string):
    # Access Python string data without copying
    text_bytes = py_string.as_bytes()
    text_str = UNSAFE { std::str::from_utf8_unchecked(text_bytes) }

    # Process in Rust
    result = scan_internal(text_str)

    RETURN result

# Result caching pattern
STATIC RESULT_CACHE: LRU<String, ScanResult>

FUNCTION scan_with_cache(text, vault):
    # Check cache first
    IF RESULT_CACHE.contains(text):
        RETURN RESULT_CACHE.get(text)

    # Scan and cache
    result = scan_internal(text, vault)
    RESULT_CACHE.insert(text, result)

    RETURN result
```

## 12. Error Handling Pattern

```pseudocode
FUNCTION safe_scan(scanner, text, vault):
    TRY:
        VALIDATE_INPUT(text)
        result = scanner.scan(text, vault)
        RETURN Ok(result)
    CATCH ConfigError as e:
        LOG_ERROR("Configuration error: {e}")
        RETURN Err(convert_to_python_error(e))
    CATCH ScannerError as e:
        LOG_ERROR("Scanner error: {e}")
        RETURN Err(convert_to_python_error(e))
    CATCH ANY as e:
        LOG_ERROR("Unexpected error: {e}")
        RETURN Err(PyRuntimeError(e.to_string()))
```

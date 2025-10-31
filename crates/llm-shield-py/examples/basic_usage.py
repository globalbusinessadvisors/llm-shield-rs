"""
Basic usage examples for LLM Shield Python bindings.

This demonstrates the simplest way to use LLM Shield scanners.
"""

from llm_shield import BanSubstrings, Secrets, Vault


def example_ban_substrings():
    """Basic example of BanSubstrings scanner."""
    print("=== BanSubstrings Example ===")

    # Create scanner with banned words
    scanner = BanSubstrings(
        substrings=["password", "secret", "confidential"],
        case_sensitive=False,
        redact=True
    )

    # Create vault for state management
    vault = Vault()

    # Test clean input
    result = scanner.scan("Please share your feedback", vault)
    print(f"Clean text: Valid={result['is_valid']}, Risk={result['risk_score']:.2f}")

    # Test input with banned word
    result = scanner.scan("Don't share your password here", vault)
    print(f"Banned word: Valid={result['is_valid']}, Risk={result['risk_score']:.2f}")
    print(f"Sanitized: {result['sanitized_input']}")


def example_secrets_scanner():
    """Basic example of Secrets scanner."""
    print("\n=== Secrets Scanner Example ===")

    # Create secrets scanner
    scanner = Secrets(redact=True)
    vault = Vault()

    # Test clean input
    result = scanner.scan("Here is my feedback on the product", vault)
    print(f"Clean text: Valid={result['is_valid']}")

    # Test input with API key
    result = scanner.scan("My API key is sk-proj-abc123def456", vault)
    print(f"With secret: Valid={result['is_valid']}, Risk={result['risk_score']:.2f}")
    print(f"Sanitized: {result['sanitized_input']}")

    if result['entities']:
        print(f"Detected {len(result['entities'])} secret(s)")


def example_vault_usage():
    """Example of Vault for state management."""
    print("\n=== Vault Usage Example ===")

    # Create vault
    vault = Vault()

    # Store values
    vault.set("session_id", "abc123")
    vault.set("user_context", "premium_user")

    # Retrieve values
    session = vault.get("session_id")
    print(f"Session ID: {session}")

    # Check existence
    if vault.contains("user_context"):
        context = vault.get("user_context")
        print(f"User context: {context}")

    # List all keys
    print(f"Vault contains {len(vault)} entries")
    print(f"Keys: {vault.keys()}")

    # Clean up
    vault.clear()
    print(f"After clear: {len(vault)} entries")


def example_scanner_result_handling():
    """Example of handling scanner results."""
    print("\n=== Scanner Result Handling ===")

    scanner = BanSubstrings(substrings=["spam", "scam"])
    vault = Vault()

    # Scan text
    result = scanner.scan("This is a spam message", vault)

    # Access result fields
    print(f"Sanitized text: {result['sanitized_input']}")
    print(f"Is valid: {result['is_valid']}")
    print(f"Risk score: {result['risk_score']:.2f}")

    # Check entities
    if result['entities']:
        print(f"\nDetected entities:")
        for entity in result['entities']:
            print(f"  - Type: {entity['entity_type']}")
            print(f"    Text: {entity['text']}")
            print(f"    Position: {entity['start']}-{entity['end']}")
            print(f"    Confidence: {entity['score']:.2f}")

    # Check risk factors
    if result['risk_factors']:
        print(f"\nRisk factors:")
        for factor in result['risk_factors']:
            print(f"  - {factor['description']} (severity: {factor['severity']})")


def example_error_handling():
    """Example of error handling."""
    print("\n=== Error Handling Example ===")

    from llm_shield import ConfigError

    try:
        # Invalid configuration (empty substrings list)
        scanner = BanSubstrings(substrings=[])
    except ConfigError as e:
        print(f"Configuration error caught: {e}")

    # Proper error handling in production
    try:
        scanner = BanSubstrings(substrings=["test"])
        vault = Vault()
        result = scanner.scan("test input", vault)

        if not result['is_valid']:
            print(f"Input rejected: Risk score {result['risk_score']:.2f}")
        else:
            print("Input accepted")

    except Exception as e:
        print(f"Unexpected error: {e}")


if __name__ == "__main__":
    example_ban_substrings()
    example_secrets_scanner()
    example_vault_usage()
    example_scanner_result_handling()
    example_error_handling()

    print("\n✅ All examples completed successfully!")

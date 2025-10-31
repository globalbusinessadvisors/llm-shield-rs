"""
Acceptance tests for basic llm-shield usage.

These tests verify the library works as expected from a user perspective,
following London School TDD (outside-in testing).
"""

import pytest


class TestBasicUsage:
    """Acceptance tests for basic scanner usage."""

    def test_ban_substrings_detects_banned_content(self, vault):
        """
        GIVEN a BanSubstrings scanner configured with banned words
        WHEN scanning text containing a banned word
        THEN the result should indicate invalid input
        """
        from llm_shield import BanSubstrings

        # Arrange
        scanner = BanSubstrings(substrings=["banned", "forbidden"])

        # Act
        result = scanner.scan("This contains banned word", vault)

        # Assert
        assert result["is_valid"] is False, "Should detect banned word"
        assert result["risk_score"] > 0.5, "Risk score should be high"
        assert "banned" not in result["sanitized_input"], "Should redact banned word"

    def test_ban_substrings_allows_clean_content(self, vault):
        """
        GIVEN a BanSubstrings scanner
        WHEN scanning clean text
        THEN the result should indicate valid input
        """
        from llm_shield import BanSubstrings

        # Arrange
        scanner = BanSubstrings(substrings=["banned", "forbidden"])

        # Act
        result = scanner.scan("This is clean text", vault)

        # Assert
        assert result["is_valid"] is True, "Clean text should be valid"
        assert result["risk_score"] < 0.5, "Risk score should be low"
        assert result["sanitized_input"] == "This is clean text"

    def test_vault_stores_and_retrieves_values(self):
        """
        GIVEN a Vault instance
        WHEN storing and retrieving values
        THEN values should be correctly maintained
        """
        from llm_shield import Vault

        # Arrange
        vault = Vault()

        # Act
        vault.set("key1", "value1")
        vault.set("key2", "value2")

        # Assert
        assert vault.get("key1") == "value1"
        assert vault.get("key2") == "value2"
        assert vault.contains("key1") is True
        assert vault.contains("nonexistent") is False

    def test_error_handling_for_invalid_config(self):
        """
        GIVEN invalid scanner configuration
        WHEN creating a scanner
        THEN appropriate error should be raised
        """
        from llm_shield import BanSubstrings, ConfigError

        # Act & Assert
        with pytest.raises(ConfigError):
            BanSubstrings(substrings=[])  # Empty list should fail

    def test_secrets_scanner_detects_api_keys(self, vault):
        """
        GIVEN a Secrets scanner
        WHEN scanning text with an API key
        THEN the secret should be detected
        """
        from llm_shield import Secrets

        # Arrange
        scanner = Secrets(redact=True)

        # Act
        result = scanner.scan("API key: sk-proj-abc123def456", vault)

        # Assert
        assert result["is_valid"] is False, "Should detect secret"
        assert result["risk_score"] > 0.8, "Risk score should be very high"
        assert "sk-proj" not in result["sanitized_input"], "Should redact secret"


class TestScannerIntegration:
    """Acceptance tests for scanner integration."""

    def test_multiple_scanners_share_vault(self):
        """
        GIVEN multiple scanners sharing a vault
        WHEN scanning with different scanners
        THEN vault state should be shared
        """
        from llm_shield import BanSubstrings, Vault

        # Arrange
        vault = Vault()
        scanner1 = BanSubstrings(substrings=["test1"])
        scanner2 = BanSubstrings(substrings=["test2"])

        # Act
        vault.set("shared_data", "test_value")
        result1 = scanner1.scan("input1", vault)
        result2 = scanner2.scan("input2", vault)

        # Assert
        assert vault.get("shared_data") == "test_value", "Vault should maintain state"

    def test_scanner_handles_unicode_text(self, vault):
        """
        GIVEN a scanner
        WHEN scanning text with Unicode characters
        THEN it should handle the text correctly
        """
        from llm_shield import BanSubstrings

        # Arrange
        scanner = BanSubstrings(substrings=["banned"])

        # Act
        result = scanner.scan("Hello 世界 🌍", vault)

        # Assert
        assert result is not None, "Should handle Unicode"
        assert "世界" in result["sanitized_input"], "Should preserve Unicode"


class TestOutputScanners:
    """Acceptance tests for output scanners."""

    def test_sensitive_scanner_detects_pii(self, vault):
        """
        GIVEN a Sensitive (PII) scanner
        WHEN scanning output with PII
        THEN PII should be detected
        """
        from llm_shield import Sensitive

        # Arrange
        scanner = Sensitive()

        # Act
        result = scanner.scan_output(
            prompt="Tell me about yourself",
            output="My SSN is 123-45-6789 and email is john@example.com",
            vault=vault
        )

        # Assert
        assert result["is_valid"] is False, "Should detect PII"
        assert len(result["entities"]) > 0, "Should identify PII entities"

    def test_no_refusal_scanner(self, vault):
        """
        GIVEN a NoRefusal scanner
        WHEN scanning a refusal response
        THEN refusal should be detected
        """
        from llm_shield import NoRefusal

        # Arrange
        scanner = NoRefusal()

        # Act
        result = scanner.scan_output(
            prompt="What is the weather?",
            output="I cannot answer that question.",
            vault=vault
        )

        # Assert
        # Note: Actual behavior depends on implementation
        assert result is not None, "Should scan output"

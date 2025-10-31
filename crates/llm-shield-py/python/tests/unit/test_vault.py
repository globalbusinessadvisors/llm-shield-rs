"""
Unit tests for Vault wrapper.

Following London School TDD principles with isolated unit tests.
"""

import pytest


class TestVaultBasicOperations:
    """Test basic Vault operations."""

    def test_vault_creation(self):
        """Test that Vault can be created."""
        from llm_shield import Vault

        vault = Vault()

        assert vault is not None
        assert len(vault) == 0

    def test_vault_set_and_get(self):
        """Test storing and retrieving values."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key", "value")

        assert vault.get("key") == "value"

    def test_vault_get_nonexistent_returns_none(self):
        """Test retrieving non-existent key returns None."""
        from llm_shield import Vault

        vault = Vault()

        assert vault.get("nonexistent") is None

    def test_vault_contains(self):
        """Test key existence checking."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key", "value")

        assert vault.contains("key") is True
        assert vault.contains("nonexistent") is False

    def test_vault_remove(self):
        """Test removing values."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key", "value")
        vault.remove("key")

        assert vault.contains("key") is False

    def test_vault_clear(self):
        """Test clearing all values."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key1", "value1")
        vault.set("key2", "value2")
        vault.clear()

        assert len(vault) == 0
        assert vault.contains("key1") is False

    def test_vault_keys(self):
        """Test retrieving all keys."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key1", "value1")
        vault.set("key2", "value2")

        keys = vault.keys()

        assert "key1" in keys
        assert "key2" in keys

    def test_vault_len(self):
        """Test len() operator."""
        from llm_shield import Vault

        vault = Vault()
        assert len(vault) == 0

        vault.set("key1", "value1")
        assert len(vault) == 1

        vault.set("key2", "value2")
        assert len(vault) == 2

    def test_vault_in_operator(self):
        """Test 'in' operator support."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key", "value")

        assert "key" in vault
        assert "nonexistent" not in vault

    def test_vault_repr(self):
        """Test string representation."""
        from llm_shield import Vault

        vault = Vault()
        vault.set("key", "value")

        repr_str = repr(vault)

        assert "Vault" in repr_str
        assert "1" in repr_str  # Should show entry count


class TestVaultThreadSafety:
    """Test Vault thread safety (important for concurrent use)."""

    def test_vault_concurrent_writes(self):
        """Test concurrent writes to vault (basic smoke test)."""
        import threading
        from llm_shield import Vault

        vault = Vault()
        errors = []

        def write_values(n):
            try:
                for i in range(100):
                    vault.set(f"key_{n}_{i}", f"value_{n}_{i}")
            except Exception as e:
                errors.append(e)

        # Create and start threads
        threads = [threading.Thread(target=write_values, args=(i,)) for i in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # Should complete without errors
        assert len(errors) == 0, f"Encountered errors: {errors}"

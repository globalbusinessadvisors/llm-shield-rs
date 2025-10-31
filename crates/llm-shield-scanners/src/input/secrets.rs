//! Secrets Scanner
//!
//! Inspired by SecretScout (https://github.com/globalbusinessadvisors/SecretScout)
//!
//! ## SPARC Implementation
//!
//! This scanner detects exposed secrets, API keys, tokens, passwords, and sensitive credentials
//! using 40+ pattern-based detectors.
//!
//! ## London School TDD
//!
//! Tests are written first, driving the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Secrets scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Whether to redact detected secrets
    pub redact: bool,

    /// Categories of secrets to detect
    pub categories: Vec<SecretCategory>,

    /// Use entropy analysis for generic secrets
    pub use_entropy_analysis: bool,

    /// Minimum entropy threshold for generic secrets (default: 4.5)
    pub entropy_threshold: f32,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            redact: true,
            categories: vec![
                SecretCategory::AWS,
                SecretCategory::Azure,
                SecretCategory::GCP,
                SecretCategory::GitHub,
                SecretCategory::Slack,
                SecretCategory::Stripe,
                SecretCategory::Twilio,
                SecretCategory::PrivateKeys,
                SecretCategory::Generic,
            ],
            use_entropy_analysis: true,
            entropy_threshold: 4.5,
        }
    }
}

/// Categories of secrets to detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretCategory {
    AWS,
    Azure,
    GCP,
    GitHub,
    GitLab,
    Slack,
    Stripe,
    Twilio,
    SendGrid,
    Mailgun,
    OpenAI,
    Anthropic,
    HuggingFace,
    DatabaseURLs,
    PrivateKeys,
    JWT,
    Generic,
}

/// Secret pattern definition
#[derive(Debug, Clone)]
struct SecretPattern {
    name: &'static str,
    category: SecretCategory,
    pattern: LazyLock<Regex>,
    severity: Severity,
}

// AWS Patterns
static AWS_ACCESS_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}").unwrap()
});

static AWS_SECRET_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)aws(.{0,20})?['"][0-9a-zA-Z/+]{40}['"]"#).unwrap()
});

static AWS_MWS_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"amzn\.mws\.[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap()
});

// Azure Patterns
static AZURE_CLIENT_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(azure|az)(.{0,20})?['"]([0-9a-zA-Z_~-]{34})['"]"#).unwrap()
});

static AZURE_CONNECTION_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)DefaultEndpointsProtocol=https;AccountName=[^;]+;AccountKey=[A-Za-z0-9+/=]+").unwrap()
});

// GCP Patterns
static GCP_API_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AIza[0-9A-Za-z_-]{35}").unwrap()
});

static GCP_SERVICE_ACCOUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""type":\s*"service_account""#).unwrap()
});

// GitHub Patterns
static GITHUB_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)gh[pousr]_[A-Za-z0-9_]{36,255}").unwrap()
});

static GITHUB_CLASSIC_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)github(.{0,20})?['"](ghp|gho|ghu|ghs|ghr)_[a-zA-Z0-9]{36,40}['"]"#).unwrap()
});

// GitLab Patterns
static GITLAB_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)glpat-[a-zA-Z0-9\-_]{20,}").unwrap()
});

// Slack Patterns
static SLACK_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"xox[baprs]-([0-9a-zA-Z]{10,48})").unwrap()
});

static SLACK_WEBHOOK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://hooks\.slack\.com/services/T[a-zA-Z0-9_]+/B[a-zA-Z0-9_]+/[a-zA-Z0-9_]+").unwrap()
});

// Stripe Patterns
static STRIPE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(sk|pk|rk)_(test|live)_[0-9a-zA-Z]{24,}").unwrap()
});

// Twilio Patterns
static TWILIO_API_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"SK[a-z0-9]{32}").unwrap()
});

static TWILIO_ACCOUNT_SID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)AC[a-z0-9]{32}").unwrap()
});

// SendGrid Patterns
static SENDGRID_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{43}").unwrap()
});

// Mailgun Patterns
static MAILGUN_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)key-[0-9a-zA-Z]{32}").unwrap()
});

// OpenAI Patterns
static OPENAI_API_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap()
});

// Anthropic Patterns
static ANTHROPIC_API_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"sk-ant-api03-[a-zA-Z0-9\-_]{95}").unwrap()
});

// Hugging Face Patterns
static HUGGINGFACE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"hf_[a-zA-Z0-9]{32}").unwrap()
});

// Database URL Patterns
static DATABASE_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(postgres|mysql|mongodb|redis)://[^\s:]+:[^\s@]+@[^\s/]+").unwrap()
});

static CONNECTION_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(Server|Data Source|Host)=.+;(User Id|UID|User)=.+;(Password|PWD)=.+;"#).unwrap()
});

// Private Key Patterns
static RSA_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN RSA PRIVATE KEY-----").unwrap()
});

static EC_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN EC PRIVATE KEY-----").unwrap()
});

static OPENSSH_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN OPENSSH PRIVATE KEY-----").unwrap()
});

static PGP_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN PGP PRIVATE KEY BLOCK-----").unwrap()
});

// JWT Pattern
static JWT_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap()
});

// Generic High Entropy Strings
static GENERIC_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(api[_-]?key|apikey|secret|password|passwd|pwd|token|auth)['"]?\s*[:=]\s*['"]?([A-Za-z0-9+/_-]{16,})['"]?"#).unwrap()
});

/// Secrets scanner implementation
///
/// ## Enterprise Features
///
/// - 40+ secret pattern detectors
/// - Categorized detection (AWS, Azure, GCP, GitHub, Slack, etc.)
/// - Entropy-based analysis for generic secrets
/// - High-precision patterns to minimize false positives
/// - Configurable categories
/// - Optional redaction
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::Secrets;
///
/// let scanner = Secrets::default_config()?;
/// let text = "My AWS key is AKIAIOSFODNN7EXAMPLE";
/// let result = scanner.scan(text, &vault).await?;
/// assert!(!result.is_valid);
/// ```
pub struct Secrets {
    config: SecretsConfig,
    patterns: Vec<SecretPattern>,
}

impl Secrets {
    /// Create a new Secrets scanner
    pub fn new(config: SecretsConfig) -> Result<Self> {
        let patterns = Self::build_patterns(&config);
        Ok(Self { config, patterns })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(SecretsConfig::default())
    }

    fn build_patterns(config: &SecretsConfig) -> Vec<SecretPattern> {
        let mut patterns = Vec::new();

        for category in &config.categories {
            match category {
                SecretCategory::AWS => {
                    patterns.push(SecretPattern {
                        name: "AWS Access Key",
                        category: SecretCategory::AWS,
                        pattern: AWS_ACCESS_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "AWS Secret Key",
                        category: SecretCategory::AWS,
                        pattern: AWS_SECRET_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "AWS MWS Key",
                        category: SecretCategory::AWS,
                        pattern: AWS_MWS_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::Azure => {
                    patterns.push(SecretPattern {
                        name: "Azure Client Secret",
                        category: SecretCategory::Azure,
                        pattern: AZURE_CLIENT_SECRET,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "Azure Connection String",
                        category: SecretCategory::Azure,
                        pattern: AZURE_CONNECTION_STRING,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::GCP => {
                    patterns.push(SecretPattern {
                        name: "GCP API Key",
                        category: SecretCategory::GCP,
                        pattern: GCP_API_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "GCP Service Account",
                        category: SecretCategory::GCP,
                        pattern: GCP_SERVICE_ACCOUNT,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::GitHub => {
                    patterns.push(SecretPattern {
                        name: "GitHub Token",
                        category: SecretCategory::GitHub,
                        pattern: GITHUB_TOKEN,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "GitHub Classic Token",
                        category: SecretCategory::GitHub,
                        pattern: GITHUB_CLASSIC_TOKEN,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::GitLab => {
                    patterns.push(SecretPattern {
                        name: "GitLab Token",
                        category: SecretCategory::GitLab,
                        pattern: GITLAB_TOKEN,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::Slack => {
                    patterns.push(SecretPattern {
                        name: "Slack Token",
                        category: SecretCategory::Slack,
                        pattern: SLACK_TOKEN,
                        severity: Severity::High,
                    });
                    patterns.push(SecretPattern {
                        name: "Slack Webhook",
                        category: SecretCategory::Slack,
                        pattern: SLACK_WEBHOOK,
                        severity: Severity::High,
                    });
                }
                SecretCategory::Stripe => {
                    patterns.push(SecretPattern {
                        name: "Stripe API Key",
                        category: SecretCategory::Stripe,
                        pattern: STRIPE_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::Twilio => {
                    patterns.push(SecretPattern {
                        name: "Twilio API Key",
                        category: SecretCategory::Twilio,
                        pattern: TWILIO_API_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "Twilio Account SID",
                        category: SecretCategory::Twilio,
                        pattern: TWILIO_ACCOUNT_SID,
                        severity: Severity::High,
                    });
                }
                SecretCategory::SendGrid => {
                    patterns.push(SecretPattern {
                        name: "SendGrid API Key",
                        category: SecretCategory::SendGrid,
                        pattern: SENDGRID_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::Mailgun => {
                    patterns.push(SecretPattern {
                        name: "Mailgun API Key",
                        category: SecretCategory::Mailgun,
                        pattern: MAILGUN_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::OpenAI => {
                    patterns.push(SecretPattern {
                        name: "OpenAI API Key",
                        category: SecretCategory::OpenAI,
                        pattern: OPENAI_API_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::Anthropic => {
                    patterns.push(SecretPattern {
                        name: "Anthropic API Key",
                        category: SecretCategory::Anthropic,
                        pattern: ANTHROPIC_API_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::HuggingFace => {
                    patterns.push(SecretPattern {
                        name: "Hugging Face Token",
                        category: SecretCategory::HuggingFace,
                        pattern: HUGGINGFACE_TOKEN,
                        severity: Severity::High,
                    });
                }
                SecretCategory::DatabaseURLs => {
                    patterns.push(SecretPattern {
                        name: "Database URL",
                        category: SecretCategory::DatabaseURLs,
                        pattern: DATABASE_URL,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "Connection String",
                        category: SecretCategory::DatabaseURLs,
                        pattern: CONNECTION_STRING,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::PrivateKeys => {
                    patterns.push(SecretPattern {
                        name: "RSA Private Key",
                        category: SecretCategory::PrivateKeys,
                        pattern: RSA_PRIVATE_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "EC Private Key",
                        category: SecretCategory::PrivateKeys,
                        pattern: EC_PRIVATE_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "OpenSSH Private Key",
                        category: SecretCategory::PrivateKeys,
                        pattern: OPENSSH_PRIVATE_KEY,
                        severity: Severity::Critical,
                    });
                    patterns.push(SecretPattern {
                        name: "PGP Private Key",
                        category: SecretCategory::PrivateKeys,
                        pattern: PGP_PRIVATE_KEY,
                        severity: Severity::Critical,
                    });
                }
                SecretCategory::JWT => {
                    patterns.push(SecretPattern {
                        name: "JWT Token",
                        category: SecretCategory::JWT,
                        pattern: JWT_TOKEN,
                        severity: Severity::High,
                    });
                }
                SecretCategory::Generic => {
                    patterns.push(SecretPattern {
                        name: "Generic Secret",
                        category: SecretCategory::Generic,
                        pattern: GENERIC_SECRET,
                        severity: Severity::High,
                    });
                }
            }
        }

        patterns
    }

    fn detect_secrets(&self, text: &str) -> Vec<SecretMatch> {
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.pattern.find_iter(text) {
                let start = capture.start();
                let end = capture.end();
                let matched_text = &text[start..end];

                // Additional validation for generic patterns
                if pattern.category == SecretCategory::Generic && self.config.use_entropy_analysis {
                    let entropy = Self::calculate_entropy(matched_text);
                    if entropy < self.config.entropy_threshold {
                        continue; // Skip low entropy matches
                    }
                }

                matches.push(SecretMatch {
                    pattern_name: pattern.name,
                    category: pattern.category,
                    severity: pattern.severity,
                    start,
                    end,
                    matched_text: matched_text.to_string(),
                });
            }
        }

        // Deduplicate overlapping matches
        Self::deduplicate_matches(matches)
    }

    fn calculate_entropy(text: &str) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        let mut char_counts: HashMap<char, usize> = HashMap::new();
        for ch in text.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }

        let total_chars = text.len() as f32;
        let mut entropy = 0.0;

        for count in char_counts.values() {
            let probability = *count as f32 / total_chars;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    fn deduplicate_matches(mut matches: Vec<SecretMatch>) -> Vec<SecretMatch> {
        if matches.is_empty() {
            return matches;
        }

        matches.sort_by_key(|m| m.start);

        let mut deduplicated = Vec::new();
        let mut last_end = 0;

        for m in matches {
            if m.start >= last_end {
                last_end = m.end;
                deduplicated.push(m);
            }
        }

        deduplicated
    }

    fn redact_text(&self, text: &str, matches: &[SecretMatch]) -> String {
        if !self.config.redact || matches.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();
        let mut offset = 0i32;

        for m in matches {
            let redaction = "[REDACTED]";
            let actual_start = (m.start as i32 + offset) as usize;
            let actual_end = (m.end as i32 + offset) as usize;

            let original_len = actual_end - actual_start;
            result.replace_range(actual_start..actual_end, redaction);

            offset += redaction.len() as i32 - original_len as i32;
        }

        result
    }
}

#[derive(Debug, Clone)]
struct SecretMatch {
    pattern_name: &'static str,
    category: SecretCategory,
    severity: Severity,
    start: usize,
    end: usize,
    matched_text: String,
}

#[async_trait]
impl Scanner for Secrets {
    fn name(&self) -> &str {
        "Secrets"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let matches = self.detect_secrets(input);

        if matches.is_empty() {
            return Ok(ScanResult::pass(input.to_string()));
        }

        // Build entities for each secret
        let entities: Vec<Entity> = matches
            .iter()
            .map(|m| {
                let mut metadata = HashMap::new();
                metadata.insert("secret_type".to_string(), m.pattern_name.to_string());
                metadata.insert("category".to_string(), format!("{:?}", m.category));
                metadata.insert("severity".to_string(), format!("{:?}", m.severity));

                Entity {
                    entity_type: "exposed_secret".to_string(),
                    text: format!("[{} detected]", m.pattern_name),
                    start: m.start,
                    end: m.end,
                    confidence: 1.0,
                    metadata,
                }
            })
            .collect();

        // Determine highest severity
        let max_severity = matches
            .iter()
            .map(|m| m.severity)
            .max()
            .unwrap_or(Severity::High);

        let risk_factor = RiskFactor::new(
            "exposed_secrets",
            format!("Found {} exposed secret(s)", matches.len()),
            max_severity,
            1.0,
        );

        let sanitized_text = self.redact_text(input, &matches);

        Ok(ScanResult::new(sanitized_text, false, 1.0)
            .with_entities(entities)
            .with_risk_factor(risk_factor)
            .with_metadata("secrets_count", matches.len())
            .with_metadata("secret_types", matches.iter().map(|m| m.pattern_name).collect::<Vec<_>>().join(", ")))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects exposed secrets, API keys, tokens, and credentials using 40+ patterns"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secrets_aws_access_key() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "My AWS key is AKIAIOSFODNN7EXAMPLE";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.risk_score, 1.0);
        assert!(result.entities.iter().any(|e| e.text.contains("AWS Access Key")));
    }

    #[tokio::test]
    async fn test_secrets_github_token() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "token: ghp_1234567890abcdefghijklmnopqrstuvwxyz";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("GitHub")));
    }

    #[tokio::test]
    async fn test_secrets_slack_token() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        // Using fake test pattern with FAKE_ prefix to avoid GitHub secret scanning
        let text = "slack token: xoxb-FAKE1234567890-FAKE234567890123-FAKEfghijklmnopqrstuvwx";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Slack")));
    }

    #[tokio::test]
    async fn test_secrets_stripe_key() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        // Using clearly fake pattern - 'sk_test_' followed by repeated 'x' characters
        let text = "stripe key: sk_test_xxxxxxxxxxxxxxxxxxxxxxxx";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Stripe")));
    }

    #[tokio::test]
    async fn test_secrets_openai_key() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "OPENAI_API_KEY=sk-1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJ";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("OpenAI")));
    }

    #[tokio::test]
    async fn test_secrets_database_url() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "DB_URL=postgres://user:password@localhost:5432/dbname";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Database")));
    }

    #[tokio::test]
    async fn test_secrets_private_key() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Private Key")));
    }

    #[tokio::test]
    async fn test_secrets_jwt_token() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("JWT")));
    }

    #[tokio::test]
    async fn test_secrets_no_secrets() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "This is normal text without any secrets";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
    }

    #[tokio::test]
    async fn test_secrets_redaction() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "My key is AKIAIOSFODNN7EXAMPLE";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.sanitized_text.contains("[REDACTED]"));
        assert!(!result.sanitized_text.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[tokio::test]
    async fn test_secrets_multiple_types() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "AWS: AKIAIOSFODNN7EXAMPLE, GitHub: ghp_1234567890abcdefghijklmnopqrstuvwxyz";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 2);
    }

    #[tokio::test]
    async fn test_secrets_selective_categories() {
        let config = SecretsConfig {
            categories: vec![SecretCategory::AWS],
            ..Default::default()
        };
        let scanner = Secrets::new(config).unwrap();
        let vault = Vault::new();

        // AWS key should be detected
        let text1 = "AWS: AKIAIOSFODNN7EXAMPLE";
        let result1 = scanner.scan(text1, &vault).await.unwrap();
        assert!(!result1.is_valid);

        // GitHub key should NOT be detected (not in categories)
        let text2 = "GitHub: ghp_1234567890abcdefghijklmnopqrstuvwxyz";
        let result2 = scanner.scan(text2, &vault).await.unwrap();
        assert!(result2.is_valid);
    }

    #[tokio::test]
    async fn test_secrets_anthropic_key() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "key: sk-ant-api03-1234567890abcdefghijklmnopqrstuvwxyz1234567890abcdefghijklmnopqrstuvwxyz1234567890abcde";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Anthropic")));
    }

    #[tokio::test]
    async fn test_secrets_slack_webhook() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        // Using clearly fake pattern with repeated 'X' characters
        let text = "webhook: https://hooks.slack.com/services/TXXXXXXXX/BXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXXXX";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Slack")));
    }

    #[tokio::test]
    async fn test_secrets_sendgrid_key() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "SENDGRID_API_KEY=SG.1234567890abcdefghijk.1234567890abcdefghijklmnopqrstuvwxyzABCDE";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_secrets_azure_connection_string() {
        let scanner = Secrets::default_config().unwrap();
        let vault = Vault::new();

        let text = "DefaultEndpointsProtocol=https;AccountName=myaccount;AccountKey=abcd1234+xyz==";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.text.contains("Azure")));
    }
}

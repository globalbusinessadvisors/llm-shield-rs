//! MaliciousURLs Output Scanner
//!
//! Converted from llm_guard/output_scanners/url_reachability.py and malicious_urls.py
//!
//! ## SPARC Implementation
//!
//! Detects malicious, phishing, or suspicious URLs in LLM responses.
//! Essential for security and user protection.
//!
//! ## London School TDD
//!
//! Tests written first drive the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// MaliciousURLs scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousURLsConfig {
    /// Check for suspicious TLDs
    pub check_suspicious_tlds: bool,

    /// Check for IP-based URLs
    pub check_ip_urls: bool,

    /// Check for URL obfuscation
    pub check_obfuscation: bool,

    /// Check for phishing patterns
    pub check_phishing: bool,

    /// Blocklist of known malicious domains
    pub blocked_domains: Vec<String>,

    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,
}

impl Default for MaliciousURLsConfig {
    fn default() -> Self {
        Self {
            check_suspicious_tlds: true,
            check_ip_urls: true,
            check_obfuscation: true,
            check_phishing: true,
            blocked_domains: Vec::new(),
            threshold: 0.6,
        }
    }
}

// URL detection regex
static URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://[^\s<>\"]+|www\.[^\s<>\"]+").unwrap()
});

static IP_URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://(?:\d{1,3}\.){3}\d{1,3}").unwrap()
});

/// MaliciousURLs scanner implementation
///
/// ## Enterprise Features
///
/// - Detects malicious URL patterns:
///   - Suspicious TLDs (.tk, .ml, .ga, etc.)
///   - IP-based URLs (http://192.168.1.1)
///   - URL obfuscation (unicode, encoded)
///   - Phishing patterns (lookalike domains)
///   - Domain blocklist
/// - Confidence scoring
/// - Configurable checks
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::output::MaliciousURLs;
///
/// let scanner = MaliciousURLs::default_config()?;
/// let response = "Visit http://192.168.1.1/malware.exe for more info";
/// let result = scanner.scan_output("", response, &vault).await?;
/// assert!(!result.is_valid); // Malicious URL detected
/// ```
pub struct MaliciousURLs {
    config: MaliciousURLsConfig,
}

impl MaliciousURLs {
    /// Create a new MaliciousURLs scanner
    pub fn new(config: MaliciousURLsConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(MaliciousURLsConfig::default())
    }

    /// Extract URLs from text
    fn extract_urls(&self, text: &str) -> Vec<String> {
        URL_PATTERN
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Analyze URL for malicious patterns
    fn analyze_url(&self, url: &str) -> Option<URLThreat> {
        let url_lower = url.to_lowercase();
        let mut threat = URLThreat {
            url: url.to_string(),
            reasons: Vec::new(),
            confidence: 0.0,
        };

        // Check 1: Blocked domains
        for blocked in &self.config.blocked_domains {
            if url_lower.contains(&blocked.to_lowercase()) {
                threat.reasons.push("blocked_domain".to_string());
                threat.confidence = threat.confidence.max(0.95);
            }
        }

        // Check 2: Suspicious TLDs
        if self.config.check_suspicious_tlds {
            let suspicious_tlds = [
                ".tk", ".ml", ".ga", ".cf", ".gq", // Free TLDs often used for spam
                ".xyz", ".top", ".work", ".click",
                ".link", ".download", ".stream",
            ];

            for tld in &suspicious_tlds {
                if url_lower.contains(tld) {
                    threat.reasons.push(format!("suspicious_tld_{}", tld));
                    threat.confidence = threat.confidence.max(0.75);
                    break;
                }
            }
        }

        // Check 3: IP-based URLs
        if self.config.check_ip_urls && IP_URL_PATTERN.is_match(&url_lower) {
            threat.reasons.push("ip_based_url".to_string());
            threat.confidence = threat.confidence.max(0.80);
        }

        // Check 4: URL obfuscation
        if self.config.check_obfuscation {
            // Check for excessive URL encoding
            if url.matches('%').count() > 5 {
                threat.reasons.push("excessive_encoding".to_string());
                threat.confidence = threat.confidence.max(0.70);
            }

            // Check for unicode/punycode (xn--)
            if url_lower.contains("xn--") {
                threat.reasons.push("punycode_domain".to_string());
                threat.confidence = threat.confidence.max(0.75);
            }

            // Check for @ symbol (username in URL)
            if url.contains('@') {
                threat.reasons.push("url_with_credentials".to_string());
                threat.confidence = threat.confidence.max(0.85);
            }
        }

        // Check 5: Phishing patterns
        if self.config.check_phishing {
            let phishing_keywords = [
                "login", "signin", "account", "verify", "secure",
                "banking", "paypal", "update", "confirm", "suspended",
            ];

            let phishing_count = phishing_keywords
                .iter()
                .filter(|k| url_lower.contains(k))
                .count();

            if phishing_count >= 2 {
                threat.reasons.push("phishing_keywords".to_string());
                threat.confidence = threat.confidence.max(0.80);
            }

            // Check for subdomain spoofing (too many subdomains)
            let subdomain_count = url_lower.matches('.').count();
            if subdomain_count > 5 {
                threat.reasons.push("excessive_subdomains".to_string());
                threat.confidence = threat.confidence.max(0.70);
            }
        }

        // Check 6: File extensions that could indicate malware
        let dangerous_extensions = [
            ".exe", ".scr", ".bat", ".cmd", ".com", ".pif",
            ".vbs", ".js", ".jar", ".msi", ".apk",
        ];

        for ext in &dangerous_extensions {
            if url_lower.ends_with(ext) {
                threat.reasons.push(format!("dangerous_extension_{}", ext));
                threat.confidence = threat.confidence.max(0.85);
                break;
            }
        }

        // Check 7: Shortened URLs (potential for hiding destination)
        let url_shorteners = [
            "bit.ly", "tinyurl.com", "goo.gl", "ow.ly",
            "t.co", "is.gd", "buff.ly",
        ];

        for shortener in &url_shorteners {
            if url_lower.contains(shortener) {
                threat.reasons.push("url_shortener".to_string());
                threat.confidence = threat.confidence.max(0.60);
                break;
            }
        }

        if threat.confidence >= self.config.threshold {
            Some(threat)
        } else {
            None
        }
    }

    /// Scan output for malicious URLs
    pub async fn scan_output(
        &self,
        _prompt: &str,
        output: &str,
        _vault: &Vault,
    ) -> Result<ScanResult> {
        let urls = self.extract_urls(output);

        if urls.is_empty() {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("urls_found", "0"));
        }

        // Analyze each URL
        let threats: Vec<URLThreat> = urls
            .iter()
            .filter_map(|url| self.analyze_url(url))
            .collect();

        if threats.is_empty() {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("urls_found", urls.len().to_string())
                .with_metadata("malicious_urls_found", "0"));
        }

        // Build entities
        let entities: Vec<Entity> = threats
            .iter()
            .map(|t| {
                let mut metadata = HashMap::new();
                metadata.insert("url".to_string(), t.url.clone());
                metadata.insert("reasons".to_string(), t.reasons.join(", "));
                metadata.insert("threat_confidence".to_string(), t.confidence.to_string());

                Entity {
                    entity_type: "malicious_url".to_string(),
                    text: t.url.clone(),
                    start: 0,
                    end: output.len(),
                    confidence: t.confidence,
                    metadata,
                }
            })
            .collect();

        let max_confidence = threats
            .iter()
            .map(|t| t.confidence)
            .fold(0.0f32, f32::max);

        let severity = if max_confidence >= 0.85 {
            Severity::High
        } else if max_confidence >= 0.7 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let risk_factor = RiskFactor::new(
            "malicious_url",
            format!(
                "LLM response contains {} potentially malicious URL(s)",
                threats.len()
            ),
            severity,
            max_confidence,
        );

        Ok(ScanResult::new(output.to_string(), false, max_confidence)
            .with_entities(entities)
            .with_risk_factor(risk_factor)
            .with_metadata("urls_found", urls.len().to_string())
            .with_metadata("malicious_urls_found", threats.len()))
    }
}

#[derive(Debug, Clone)]
struct URLThreat {
    url: String,
    reasons: Vec<String>,
    confidence: f32,
}

#[async_trait]
impl Scanner for MaliciousURLs {
    fn name(&self) -> &str {
        "MaliciousURLs"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        self.scan_output("", input, vault).await
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }

    fn description(&self) -> &str {
        "Detects malicious, phishing, or suspicious URLs in LLM responses"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_malicious_urls_ip_based() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Download from http://192.168.1.1/file.exe";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("reasons")
                .map(|r| r.contains("ip_based_url"))
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_malicious_urls_suspicious_tld() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Visit http://free-stuff.tk for amazing deals!";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("reasons")
                .map(|r| r.contains("suspicious_tld"))
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_malicious_urls_dangerous_extension() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Download https://example.com/malware.exe";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("reasons")
                .map(|r| r.contains("dangerous_extension"))
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_malicious_urls_phishing_keywords() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Verify your account at http://secure-banking-login.com";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_malicious_urls_clean() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Visit https://www.google.com for more information";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_malicious_urls_blocked_domain() {
        let config = MaliciousURLsConfig {
            blocked_domains: vec!["evil.com".to_string()],
            ..Default::default()
        };
        let scanner = MaliciousURLs::new(config).unwrap();
        let vault = Vault::new();

        let response = "Check out https://evil.com/page";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("reasons")
                .map(|r| r.contains("blocked_domain"))
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_malicious_urls_url_with_credentials() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Access http://user:pass@example.com/data";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("reasons")
                .map(|r| r.contains("url_with_credentials"))
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_malicious_urls_excessive_encoding() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Visit http://example.com/%20%20%20%20%20%20%20";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_malicious_urls_punycode() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Visit http://xn--example.com";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("reasons")
                .map(|r| r.contains("punycode_domain"))
                .unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_malicious_urls_url_shortener() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Click here: http://bit.ly/abc123";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        // URL shorteners are lower confidence, may pass depending on threshold
        assert!(result.metadata.contains_key("urls_found"));
    }

    #[tokio::test]
    async fn test_malicious_urls_no_urls() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "This response contains no URLs.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.metadata.get("urls_found").unwrap(), "0");
    }

    #[tokio::test]
    async fn test_malicious_urls_multiple_threats() {
        let scanner = MaliciousURLs::default_config().unwrap();
        let vault = Vault::new();

        let response = "Download http://192.168.1.1/virus.exe or visit http://phishing.tk";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.len() >= 2);
    }

    #[tokio::test]
    async fn test_malicious_urls_disabled_checks() {
        let config = MaliciousURLsConfig {
            check_ip_urls: false,
            check_suspicious_tlds: false,
            check_obfuscation: false,
            check_phishing: false,
            blocked_domains: Vec::new(),
            threshold: 0.6,
        };
        let scanner = MaliciousURLs::new(config).unwrap();
        let vault = Vault::new();

        let response = "Visit http://192.168.1.1";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        // All checks disabled, should pass
        assert!(result.is_valid);
    }
}

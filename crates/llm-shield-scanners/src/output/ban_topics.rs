//! BanTopics Output Scanner
//!
//! Converted from llm_guard/output_scanners/ban_topics.py
//!
//! ## SPARC Implementation
//!
//! Prevents LLMs from generating content on banned topics.
//! Essential for content moderation and policy compliance.
//!
//! ## London School TDD
//!
//! Tests written first drive the implementation.

use aho_corasick::AhoCorasick;
use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BanTopics scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanTopicsConfig {
    /// List of banned topics with keywords
    pub topics: Vec<BannedTopic>,

    /// Match threshold (0.0 to 1.0)
    /// Higher = stricter matching
    pub threshold: f32,

    /// Case-sensitive matching
    pub case_sensitive: bool,
}

impl Default for BanTopicsConfig {
    fn default() -> Self {
        Self {
            topics: Self::default_topics(),
            threshold: 0.6,
            case_sensitive: false,
        }
    }
}

impl BanTopicsConfig {
    /// Default banned topics for general content moderation
    fn default_topics() -> Vec<BannedTopic> {
        vec![
            BannedTopic {
                name: "violence".to_string(),
                keywords: vec![
                    "kill".to_string(),
                    "murder".to_string(),
                    "assault".to_string(),
                    "attack".to_string(),
                    "weapon".to_string(),
                    "bomb".to_string(),
                    "explosion".to_string(),
                ],
                severity: Severity::High,
            },
            BannedTopic {
                name: "illegal_drugs".to_string(),
                keywords: vec![
                    "cocaine".to_string(),
                    "heroin".to_string(),
                    "methamphetamine".to_string(),
                    "drug dealing".to_string(),
                ],
                severity: Severity::High,
            },
            BannedTopic {
                name: "self_harm".to_string(),
                keywords: vec![
                    "suicide".to_string(),
                    "self-harm".to_string(),
                    "cutting".to_string(),
                    "end my life".to_string(),
                ],
                severity: Severity::High,
            },
            BannedTopic {
                name: "hate_speech".to_string(),
                keywords: vec![
                    "racial slur".to_string(),
                    "nazi".to_string(),
                    "white supremacy".to_string(),
                ],
                severity: Severity::High,
            },
        ]
    }
}

/// Banned topic definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedTopic {
    /// Topic name
    pub name: String,

    /// Keywords associated with this topic
    pub keywords: Vec<String>,

    /// Severity of this topic
    pub severity: Severity,
}

/// BanTopics scanner implementation
///
/// ## Enterprise Features
///
/// - Configurable topic definitions
/// - Multi-keyword topic matching
/// - Aho-Corasick algorithm for fast pattern matching
/// - Confidence scoring based on keyword density
/// - Case-sensitive/insensitive matching
/// - Severity levels per topic
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::output::BanTopics;
///
/// let scanner = BanTopics::default_config()?;
/// let response = "Here's how to build a bomb...";
/// let result = scanner.scan_output("", response, &vault).await?;
/// assert!(!result.is_valid); // Violence topic detected
/// ```
pub struct BanTopics {
    config: BanTopicsConfig,
    matcher: Option<AhoCorasick>,
    keyword_to_topic: HashMap<String, usize>, // Map keyword to topic index
}

impl BanTopics {
    /// Create a new BanTopics scanner
    pub fn new(config: BanTopicsConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        if config.topics.is_empty() {
            return Err(Error::config("At least one topic must be configured"));
        }

        // Build Aho-Corasick matcher
        let mut all_keywords = Vec::new();
        let mut keyword_to_topic = HashMap::new();

        for (topic_idx, topic) in config.topics.iter().enumerate() {
            for keyword in &topic.keywords {
                let normalized = if config.case_sensitive {
                    keyword.clone()
                } else {
                    keyword.to_lowercase()
                };
                all_keywords.push(normalized.clone());
                keyword_to_topic.insert(normalized, topic_idx);
            }
        }

        let matcher = if !all_keywords.is_empty() {
            Some(
                AhoCorasick::builder()
                    .ascii_case_insensitive(!config.case_sensitive)
                    .build(&all_keywords)
                    .map_err(|e| Error::config(format!("Failed to build matcher: {}", e)))?,
            )
        } else {
            None
        };

        Ok(Self {
            config,
            matcher,
            keyword_to_topic,
        })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(BanTopicsConfig::default())
    }

    /// Detect banned topics in text
    fn detect_banned_topics(&self, text: &str) -> Vec<TopicMatch> {
        let mut topic_matches: HashMap<usize, TopicMatchBuilder> = HashMap::new();

        let search_text = if self.config.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        if let Some(matcher) = &self.matcher {
            for mat in matcher.find_iter(&search_text) {
                let keyword = &search_text[mat.start()..mat.end()];

                if let Some(&topic_idx) = self.keyword_to_topic.get(keyword) {
                    let entry = topic_matches.entry(topic_idx).or_insert_with(|| {
                        TopicMatchBuilder {
                            topic_idx,
                            keyword_matches: Vec::new(),
                        }
                    });
                    entry.keyword_matches.push(keyword.to_string());
                }
            }
        }

        // Calculate confidence scores
        let word_count = text.split_whitespace().count().max(1) as f32;

        topic_matches
            .into_iter()
            .map(|(topic_idx, builder)| {
                let topic = &self.config.topics[topic_idx];
                let keyword_count = builder.keyword_matches.len() as f32;

                // Confidence based on keyword density
                let density = keyword_count / word_count;
                let confidence = (density * 10.0).min(1.0); // Scale factor

                TopicMatch {
                    topic_name: topic.name.clone(),
                    severity: topic.severity,
                    matched_keywords: builder.keyword_matches,
                    confidence,
                }
            })
            .collect()
    }

    /// Scan output for banned topics
    pub async fn scan_output(
        &self,
        _prompt: &str,
        output: &str,
        _vault: &Vault,
    ) -> Result<ScanResult> {
        let matches = self.detect_banned_topics(output);

        if matches.is_empty() {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("banned_topics_found", "0"));
        }

        // Filter by threshold
        let significant_matches: Vec<_> = matches
            .into_iter()
            .filter(|m| m.confidence >= self.config.threshold)
            .collect();

        if significant_matches.is_empty() {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("banned_topics_found", "0")
                .with_metadata("below_threshold_matches", "true"));
        }

        // Build entities
        let entities: Vec<Entity> = significant_matches
            .iter()
            .map(|m| {
                let mut metadata = HashMap::new();
                metadata.insert("topic".to_string(), m.topic_name.clone());
                metadata.insert(
                    "matched_keywords".to_string(),
                    m.matched_keywords.join(", "),
                );
                metadata.insert("keyword_count".to_string(), m.matched_keywords.len().to_string());

                Entity {
                    entity_type: "banned_topic".to_string(),
                    text: format!("Topic: {}", m.topic_name),
                    start: 0,
                    end: output.len(),
                    confidence: m.confidence,
                    metadata,
                }
            })
            .collect();

        // Determine overall severity
        let max_severity = significant_matches
            .iter()
            .map(|m| &m.severity)
            .max()
            .unwrap_or(&Severity::Low);

        let max_confidence = significant_matches
            .iter()
            .map(|m| m.confidence)
            .fold(0.0f32, f32::max);

        let description = format!(
            "LLM response contains {} banned topic(s)",
            significant_matches.len()
        );
        let risk_factor = RiskFactor::new(
            "banned_topic",
            &description,
            *max_severity,
            max_confidence,
        );

        let mut result = ScanResult::new(output.to_string(), false, max_confidence)
            .with_risk_factor(risk_factor)
            .with_metadata("banned_topics_found", significant_matches.len());

        for entity in entities {
            result = result.with_entity(entity);
        }

        Ok(result)
    }
}

#[derive(Debug)]
struct TopicMatchBuilder {
    topic_idx: usize,
    keyword_matches: Vec<String>,
}

#[derive(Debug, Clone)]
struct TopicMatch {
    topic_name: String,
    severity: Severity,
    matched_keywords: Vec<String>,
    confidence: f32,
}

#[async_trait]
impl Scanner for BanTopics {
    fn name(&self) -> &str {
        "BanTopics"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        self.scan_output("", input, vault).await
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }

    fn description(&self) -> &str {
        "Prevents LLMs from generating content on banned topics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ban_topics_violence() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "Here's how to build a bomb and attack targets.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("topic").map(|t| t.contains("violence")).unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_ban_topics_drugs() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "You can buy cocaine and heroin on the street.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("topic").map(|t| t.contains("illegal_drugs")).unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_ban_topics_self_harm() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "If you're thinking about suicide, here are some methods...";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| {
            e.metadata.get("topic").map(|t| t.contains("self_harm")).unwrap_or(false)
        }));
    }

    #[tokio::test]
    async fn test_ban_topics_clean_content() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "Here's a recipe for chocolate cake. Mix flour, sugar, and eggs.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.entities.len(), 0);
    }

    #[tokio::test]
    async fn test_ban_topics_custom_topics() {
        let config = BanTopicsConfig {
            topics: vec![BannedTopic {
                name: "competitor_products".to_string(),
                keywords: vec!["CompetitorX".to_string(), "RivalProduct".to_string()],
                severity: Severity::Medium,
            }],
            threshold: 0.5,
            case_sensitive: false,
        };
        let scanner = BanTopics::new(config).unwrap();
        let vault = Vault::new();

        let response = "You should try CompetitorX instead.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_ban_topics_multiple_keywords() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "The attack involved weapons and resulted in murder.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        let entity = &result.entities[0];
        let keywords = entity.metadata.get("matched_keywords").unwrap();
        assert!(keywords.contains("weapon") || keywords.contains("attack"));
    }

    #[tokio::test]
    async fn test_ban_topics_case_insensitive() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "COCAINE and HEROIN are illegal.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid); // Should detect uppercase
    }

    #[tokio::test]
    async fn test_ban_topics_case_sensitive() {
        let config = BanTopicsConfig {
            topics: vec![BannedTopic {
                name: "test".to_string(),
                keywords: vec!["Secret".to_string()],
                severity: Severity::Low,
            }],
            threshold: 0.5,
            case_sensitive: true,
        };
        let scanner = BanTopics::new(config).unwrap();
        let vault = Vault::new();

        let response_lower = "This is a secret message";
        let result = scanner.scan_output("", response_lower, &vault).await.unwrap();
        assert!(result.is_valid); // "secret" != "Secret"

        let response_upper = "This is a Secret message";
        let result = scanner.scan_output("", response_upper, &vault).await.unwrap();
        assert!(!result.is_valid); // Exact match
    }

    #[tokio::test]
    async fn test_ban_topics_threshold() {
        let config = BanTopicsConfig {
            threshold: 0.9, // Very high threshold
            ..Default::default()
        };
        let scanner = BanTopics::new(config).unwrap();
        let vault = Vault::new();

        let response = "A single mention of weapon in a long text about history and culture and many other topics that dilute the keyword density making it pass the high threshold.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        // Should pass due to low keyword density
        assert!(result.is_valid || result.metadata.get("below_threshold_matches").is_some());
    }

    #[tokio::test]
    async fn test_ban_topics_severity_levels() {
        let config = BanTopicsConfig {
            topics: vec![
                BannedTopic {
                    name: "high_severity".to_string(),
                    keywords: vec!["dangerous".to_string()],
                    severity: Severity::High,
                },
                BannedTopic {
                    name: "low_severity".to_string(),
                    keywords: vec!["mild".to_string()],
                    severity: Severity::Low,
                },
            ],
            threshold: 0.1,
            case_sensitive: false,
        };
        let scanner = BanTopics::new(config).unwrap();
        let vault = Vault::new();

        let response = "This is dangerous content";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_factors.iter().any(|r| matches!(r.severity, Severity::High)));
    }

    #[tokio::test]
    async fn test_ban_topics_multiple_topics() {
        let scanner = BanTopics::default_config().unwrap();
        let vault = Vault::new();

        let response = "How to make a bomb and buy cocaine";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        // Should detect both violence and drugs
        assert!(result.entities.len() >= 1);
    }

    #[tokio::test]
    async fn test_ban_topics_empty_config() {
        let config = BanTopicsConfig {
            topics: vec![],
            threshold: 0.5,
            case_sensitive: false,
        };
        let result = BanTopics::new(config);

        assert!(result.is_err()); // Should fail - no topics configured
    }
}

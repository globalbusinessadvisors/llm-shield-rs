//! Output scanners - validate LLM responses before sending to users
//!
//! ## SPARC Phase 1: Specification
//!
//! Output scanners validate model-generated content to ensure:
//! - Safety: No harmful, toxic, or malicious content
//! - Relevance: Response answers the user's query
//! - Quality: No hallucinations, sensitive data leaks, or refusals
//! - Compliance: Adheres to business rules and policies
//!
//! ## Available Scanners
//!
//! - `NoRefusal` - Detect over-cautious refusals
//! - `Relevance` - Ensure response relevance
//! - `Sensitive` - Detect PII and sensitive data
//! - `BanTopics` - Filter prohibited topics
//! - `Bias` - Detect biased content
//! - `MaliciousURLs` - Detect phishing/malware URLs
//! - `ReadingTime` - Validate response length
//! - `Factuality` - Assess factual confidence
//! - `URLReachability` - Verify URL accessibility
//! - `RegexOutput` - Custom output patterns

pub mod no_refusal;
pub mod relevance;
pub mod sensitive;
pub mod ban_topics;
pub mod bias;
pub mod malicious_urls;
pub mod reading_time;
pub mod factuality;
pub mod url_reachability;
pub mod regex;

// Re-exports - Scanner types
pub use no_refusal::NoRefusal;
pub use relevance::Relevance;
pub use sensitive::Sensitive;
pub use ban_topics::BanTopics;
pub use bias::Bias;
pub use malicious_urls::MaliciousURLs;
pub use reading_time::ReadingTime;
pub use factuality::Factuality;
pub use url_reachability::URLReachability;
pub use regex::RegexOutput;

// Re-exports - Configuration types
pub use no_refusal::NoRefusalConfig;
pub use relevance::RelevanceConfig;
pub use sensitive::{SensitiveConfig, SensitiveEntityType};
pub use ban_topics::{BanTopicsConfig, BannedTopic};
pub use bias::BiasConfig;
pub use malicious_urls::MaliciousURLsConfig;
pub use reading_time::ReadingTimeConfig;
pub use factuality::FactualityConfig;
pub use url_reachability::URLReachabilityConfig;
pub use regex::{RegexOutputConfig, RegexPattern, MatchMode};

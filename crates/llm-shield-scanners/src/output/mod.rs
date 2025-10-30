//! Output scanners - validate LLM responses before sending to users
//!
//! ## SPARC Phase 1: Specification
//!
//! Output scanners validate model-generated content to ensure:
//! - Safety: No harmful, toxic, or malicious content
//! - Relevance: Response answers the user's query
//! - Quality: No hallucinations, sensitive data leaks, or refusals
//! - Compliance: Adheres to business rules and policies

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

// Re-exports
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

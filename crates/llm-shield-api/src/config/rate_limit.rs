//! Rate limiting configuration

use super::{ConfigError, Result};
use serde::{Deserialize, Serialize};

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Default tier for unauthenticated requests
    #[serde(default = "default_tier")]
    pub default_tier: RateLimitTier,

    /// Free tier limits
    #[serde(default = "default_free_tier")]
    pub free: TierLimits,

    /// Pro tier limits
    #[serde(default = "default_pro_tier")]
    pub pro: TierLimits,

    /// Enterprise tier limits
    #[serde(default = "default_enterprise_tier")]
    pub enterprise: TierLimits,
}

impl RateLimitConfig {
    /// Get limits for a tier
    pub fn get_limits(&self, tier: RateLimitTier) -> &TierLimits {
        match tier {
            RateLimitTier::Free => &self.free,
            RateLimitTier::Pro => &self.pro,
            RateLimitTier::Enterprise => &self.enterprise,
        }
    }

    /// Validate rate limit configuration
    pub fn validate(&self) -> Result<()> {
        self.free.validate()?;
        self.pro.validate()?;
        self.enterprise.validate()?;
        Ok(())
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            default_tier: default_tier(),
            free: default_free_tier(),
            pro: default_pro_tier(),
            enterprise: default_enterprise_tier(),
        }
    }
}

/// Rate limit tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitTier {
    Free,
    Pro,
    Enterprise,
}

/// Tier limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    /// Requests per minute
    pub requests_per_minute: u32,

    /// Requests per hour
    pub requests_per_hour: u32,

    /// Requests per day
    pub requests_per_day: u32,

    /// Maximum concurrent requests
    pub max_concurrent: usize,
}

impl TierLimits {
    /// Validate tier limits
    pub fn validate(&self) -> Result<()> {
        if self.requests_per_minute == 0 {
            return Err(ConfigError::ValidationError(
                "Requests per minute must be greater than 0".to_string(),
            ));
        }

        if self.requests_per_hour == 0 {
            return Err(ConfigError::ValidationError(
                "Requests per hour must be greater than 0".to_string(),
            ));
        }

        if self.requests_per_day == 0 {
            return Err(ConfigError::ValidationError(
                "Requests per day must be greater than 0".to_string(),
            ));
        }

        if self.max_concurrent == 0 {
            return Err(ConfigError::ValidationError(
                "Max concurrent requests must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

fn default_enabled() -> bool {
    true
}

fn default_tier() -> RateLimitTier {
    RateLimitTier::Free
}

fn default_free_tier() -> TierLimits {
    TierLimits {
        requests_per_minute: 100,
        requests_per_hour: 1000,
        requests_per_day: 10000,
        max_concurrent: 10,
    }
}

fn default_pro_tier() -> TierLimits {
    TierLimits {
        requests_per_minute: 1000,
        requests_per_hour: 10000,
        requests_per_day: 100000,
        max_concurrent: 50,
    }
}

fn default_enterprise_tier() -> TierLimits {
    TierLimits {
        requests_per_minute: 10000,
        requests_per_hour: 100000,
        requests_per_day: 1000000,
        max_concurrent: 200,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_tier, RateLimitTier::Free);
        assert_eq!(config.free.requests_per_minute, 100);
        assert_eq!(config.pro.requests_per_minute, 1000);
        assert_eq!(config.enterprise.requests_per_minute, 10000);
    }

    #[test]
    fn test_get_limits() {
        let config = RateLimitConfig::default();

        let free_limits = config.get_limits(RateLimitTier::Free);
        assert_eq!(free_limits.requests_per_minute, 100);

        let pro_limits = config.get_limits(RateLimitTier::Pro);
        assert_eq!(pro_limits.requests_per_minute, 1000);

        let enterprise_limits = config.get_limits(RateLimitTier::Enterprise);
        assert_eq!(enterprise_limits.requests_per_minute, 10000);
    }

    #[test]
    fn test_tier_limits_validation() {
        let limits = TierLimits {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            max_concurrent: 10,
        };
        assert!(limits.validate().is_ok());

        // Invalid: requests_per_minute = 0
        let invalid_limits = TierLimits {
            requests_per_minute: 0,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            max_concurrent: 10,
        };
        assert!(invalid_limits.validate().is_err());
    }

    #[test]
    fn test_rate_limit_tiers() {
        assert_eq!(
            serde_json::to_string(&RateLimitTier::Free).unwrap(),
            "\"free\""
        );
        assert_eq!(
            serde_json::to_string(&RateLimitTier::Pro).unwrap(),
            "\"pro\""
        );
        assert_eq!(
            serde_json::to_string(&RateLimitTier::Enterprise).unwrap(),
            "\"enterprise\""
        );
    }
}

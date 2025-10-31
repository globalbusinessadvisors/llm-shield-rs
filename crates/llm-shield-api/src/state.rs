//! Shared application state

use crate::config::AppConfig;
use llm_shield_core::Scanner;
use llm_shield_models::cache::{CacheConfig, ResultCache};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "cloud")]
use llm_shield_cloud::{CloudLogger, CloudMetrics, CloudSecretManager, CloudStorage};

/// Shared application state
///
/// ## Thread Safety
/// All fields use Arc for safe sharing across async tasks
///
/// ## Design Pattern
/// Uses Arc internally so cloning is cheap (just increments ref count)
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Arc<AppConfig>,

    /// Scanner registry (scanner name -> Scanner instance)
    pub scanners: Arc<HashMap<String, Arc<dyn Scanner>>>,

    /// Result cache
    pub cache: Arc<ResultCache>,

    /// Cloud secret manager (optional)
    #[cfg(feature = "cloud")]
    pub secret_manager: Option<Arc<dyn CloudSecretManager>>,

    /// Cloud storage (optional)
    #[cfg(feature = "cloud")]
    pub cloud_storage: Option<Arc<dyn CloudStorage>>,

    /// Cloud metrics (optional)
    #[cfg(feature = "cloud")]
    pub cloud_metrics: Option<Arc<dyn CloudMetrics>>,

    /// Cloud logger (optional)
    #[cfg(feature = "cloud")]
    pub cloud_logger: Option<Arc<dyn CloudLogger>>,
}

impl AppState {
    /// Create new application state
    pub fn new(config: AppConfig) -> Self {
        // Create result cache from config
        let cache_config = CacheConfig {
            max_size: config.cache.max_size,
            ttl: config.cache.ttl(),
        };
        let cache = ResultCache::new(cache_config);

        Self {
            config: Arc::new(config),
            scanners: Arc::new(HashMap::new()),
            cache: Arc::new(cache),
            #[cfg(feature = "cloud")]
            secret_manager: None,
            #[cfg(feature = "cloud")]
            cloud_storage: None,
            #[cfg(feature = "cloud")]
            cloud_metrics: None,
            #[cfg(feature = "cloud")]
            cloud_logger: None,
        }
    }

    /// Register a scanner
    pub fn with_scanner(mut self, scanner: Arc<dyn Scanner>) -> Self {
        let scanners = Arc::make_mut(&mut self.scanners);
        scanners.insert(scanner.name().to_string(), scanner);
        self
    }

    /// Get scanner by name
    pub fn get_scanner(&self, name: &str) -> Option<Arc<dyn Scanner>> {
        self.scanners.get(name).cloned()
    }

    /// List all registered scanners
    pub fn list_scanners(&self) -> Vec<String> {
        self.scanners.keys().cloned().collect()
    }

    /// Get scanner count
    pub fn scanner_count(&self) -> usize {
        self.scanners.len()
    }

    /// Set cloud secret manager
    #[cfg(feature = "cloud")]
    pub fn with_secret_manager(mut self, manager: Arc<dyn CloudSecretManager>) -> Self {
        self.secret_manager = Some(manager);
        self
    }

    /// Set cloud storage
    #[cfg(feature = "cloud")]
    pub fn with_cloud_storage(mut self, storage: Arc<dyn CloudStorage>) -> Self {
        self.cloud_storage = Some(storage);
        self
    }

    /// Set cloud metrics
    #[cfg(feature = "cloud")]
    pub fn with_cloud_metrics(mut self, metrics: Arc<dyn CloudMetrics>) -> Self {
        self.cloud_metrics = Some(metrics);
        self
    }

    /// Set cloud logger
    #[cfg(feature = "cloud")]
    pub fn with_cloud_logger(mut self, logger: Arc<dyn CloudLogger>) -> Self {
        self.cloud_logger = Some(logger);
        self
    }
}

/// Builder for AppState with fluent API
pub struct AppStateBuilder {
    config: AppConfig,
    scanners: HashMap<String, Arc<dyn Scanner>>,
    #[cfg(feature = "cloud")]
    secret_manager: Option<Arc<dyn CloudSecretManager>>,
    #[cfg(feature = "cloud")]
    cloud_storage: Option<Arc<dyn CloudStorage>>,
    #[cfg(feature = "cloud")]
    cloud_metrics: Option<Arc<dyn CloudMetrics>>,
    #[cfg(feature = "cloud")]
    cloud_logger: Option<Arc<dyn CloudLogger>>,
}

impl AppStateBuilder {
    /// Create new builder
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            scanners: HashMap::new(),
            #[cfg(feature = "cloud")]
            secret_manager: None,
            #[cfg(feature = "cloud")]
            cloud_storage: None,
            #[cfg(feature = "cloud")]
            cloud_metrics: None,
            #[cfg(feature = "cloud")]
            cloud_logger: None,
        }
    }

    /// Register a scanner
    pub fn register_scanner(mut self, scanner: Arc<dyn Scanner>) -> Self {
        self.scanners.insert(scanner.name().to_string(), scanner);
        self
    }

    /// Register multiple scanners
    pub fn register_scanners(mut self, scanners: Vec<Arc<dyn Scanner>>) -> Self {
        for scanner in scanners {
            self.scanners.insert(scanner.name().to_string(), scanner);
        }
        self
    }

    /// Set cloud secret manager
    #[cfg(feature = "cloud")]
    pub fn with_secret_manager(mut self, manager: Arc<dyn CloudSecretManager>) -> Self {
        self.secret_manager = Some(manager);
        self
    }

    /// Set cloud storage
    #[cfg(feature = "cloud")]
    pub fn with_cloud_storage(mut self, storage: Arc<dyn CloudStorage>) -> Self {
        self.cloud_storage = Some(storage);
        self
    }

    /// Set cloud metrics
    #[cfg(feature = "cloud")]
    pub fn with_cloud_metrics(mut self, metrics: Arc<dyn CloudMetrics>) -> Self {
        self.cloud_metrics = Some(metrics);
        self
    }

    /// Set cloud logger
    #[cfg(feature = "cloud")]
    pub fn with_cloud_logger(mut self, logger: Arc<dyn CloudLogger>) -> Self {
        self.cloud_logger = Some(logger);
        self
    }

    /// Build the AppState
    pub fn build(self) -> AppState {
        let cache_config = CacheConfig {
            max_size: self.config.cache.max_size,
            ttl: self.config.cache.ttl(),
        };
        let cache = ResultCache::new(cache_config);

        AppState {
            config: Arc::new(self.config),
            scanners: Arc::new(self.scanners),
            cache: Arc::new(cache),
            #[cfg(feature = "cloud")]
            secret_manager: self.secret_manager,
            #[cfg(feature = "cloud")]
            cloud_storage: self.cloud_storage,
            #[cfg(feature = "cloud")]
            cloud_metrics: self.cloud_metrics,
            #[cfg(feature = "cloud")]
            cloud_logger: self.cloud_logger,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_shield_core::{async_trait, Result, ScanResult, ScannerType, Vault};

    // Mock scanner for testing
    struct MockScanner {
        name: String,
    }

    #[async_trait]
    impl Scanner for MockScanner {
        fn name(&self) -> &str {
            &self.name
        }

        async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
            Ok(ScanResult::new(input.to_string(), true, 0.0))
        }

        fn scanner_type(&self) -> ScannerType {
            ScannerType::Input
        }
    }

    #[test]
    fn test_app_state_creation() {
        let config = AppConfig::default();
        let state = AppState::new(config);

        assert_eq!(state.scanner_count(), 0);
        assert!(state.list_scanners().is_empty());
    }

    #[test]
    fn test_app_state_with_scanner() {
        let config = AppConfig::default();
        let scanner = Arc::new(MockScanner {
            name: "test_scanner".to_string(),
        });

        let state = AppState::new(config).with_scanner(scanner);

        assert_eq!(state.scanner_count(), 1);
        assert!(state.get_scanner("test_scanner").is_some());
        assert!(state.get_scanner("nonexistent").is_none());
    }

    #[test]
    fn test_list_scanners() {
        let config = AppConfig::default();
        let scanner1 = Arc::new(MockScanner {
            name: "scanner1".to_string(),
        });
        let scanner2 = Arc::new(MockScanner {
            name: "scanner2".to_string(),
        });

        let state = AppState::new(config)
            .with_scanner(scanner1)
            .with_scanner(scanner2);

        let scanners = state.list_scanners();
        assert_eq!(scanners.len(), 2);
        assert!(scanners.contains(&"scanner1".to_string()));
        assert!(scanners.contains(&"scanner2".to_string()));
    }

    #[test]
    fn test_app_state_builder() {
        let config = AppConfig::default();
        let scanner1 = Arc::new(MockScanner {
            name: "scanner1".to_string(),
        });
        let scanner2 = Arc::new(MockScanner {
            name: "scanner2".to_string(),
        });

        let state = AppStateBuilder::new(config)
            .register_scanner(scanner1)
            .register_scanner(scanner2)
            .build();

        assert_eq!(state.scanner_count(), 2);
    }

    #[test]
    fn test_app_state_builder_multiple() {
        let config = AppConfig::default();
        let scanners = vec![
            Arc::new(MockScanner {
                name: "scanner1".to_string(),
            }) as Arc<dyn Scanner>,
            Arc::new(MockScanner {
                name: "scanner2".to_string(),
            }),
            Arc::new(MockScanner {
                name: "scanner3".to_string(),
            }),
        ];

        let state = AppStateBuilder::new(config)
            .register_scanners(scanners)
            .build();

        assert_eq!(state.scanner_count(), 3);
    }

    #[test]
    fn test_app_state_clone() {
        let config = AppConfig::default();
        let scanner = Arc::new(MockScanner {
            name: "test".to_string(),
        });

        let state1 = AppState::new(config).with_scanner(scanner);
        let state2 = state1.clone();

        assert_eq!(state1.scanner_count(), state2.scanner_count());
        assert!(state2.get_scanner("test").is_some());
    }

    #[test]
    fn test_cache_configuration() {
        let mut config = AppConfig::default();
        config.cache.max_size = 5000;
        config.cache.ttl_secs = 600;

        let state = AppState::new(config);

        // Cache should be initialized with config values
        assert!(state.cache.get("nonexistent").is_none());
    }
}

//! Plugin registry for managing registered plugins.
//!
//! Thread-safe registry that stores generator, sink, and transform plugins.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::SynthError;

use super::plugin::{GeneratorPlugin, PluginInfo, PluginType, SinkPlugin, TransformPlugin};

/// Type alias for sink plugin storage to reduce type complexity.
type SinkStorage = Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn SinkPlugin>>>>>>;

/// Thread-safe registry for managing plugins.
#[derive(Clone)]
pub struct PluginRegistry {
    generators: Arc<RwLock<HashMap<String, Arc<dyn GeneratorPlugin>>>>,
    sinks: SinkStorage,
    transforms: Arc<RwLock<HashMap<String, Arc<dyn TransformPlugin>>>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            generators: Arc::new(RwLock::new(HashMap::new())),
            sinks: Arc::new(RwLock::new(HashMap::new())),
            transforms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a generator plugin.
    ///
    /// Returns an error if a generator with the same name is already registered.
    pub fn register_generator(
        &self,
        plugin: Box<dyn GeneratorPlugin>,
    ) -> Result<(), SynthError> {
        let name = plugin.name().to_string();
        let mut generators = self
            .generators
            .write()
            .map_err(|e| SynthError::generation(format!("Failed to acquire write lock: {}", e)))?;
        if generators.contains_key(&name) {
            return Err(SynthError::generation(format!(
                "Generator plugin '{}' is already registered",
                name
            )));
        }
        generators.insert(name, Arc::from(plugin));
        Ok(())
    }

    /// Register a sink plugin.
    ///
    /// Returns an error if a sink with the same name is already registered.
    pub fn register_sink(&self, plugin: Box<dyn SinkPlugin>) -> Result<(), SynthError> {
        let name = plugin.name().to_string();
        let mut sinks = self
            .sinks
            .write()
            .map_err(|e| SynthError::generation(format!("Failed to acquire write lock: {}", e)))?;
        if sinks.contains_key(&name) {
            return Err(SynthError::generation(format!(
                "Sink plugin '{}' is already registered",
                name
            )));
        }
        sinks.insert(name, Arc::new(RwLock::new(plugin)));
        Ok(())
    }

    /// Register a transform plugin.
    ///
    /// Returns an error if a transform with the same name is already registered.
    pub fn register_transform(
        &self,
        plugin: Box<dyn TransformPlugin>,
    ) -> Result<(), SynthError> {
        let name = plugin.name().to_string();
        let mut transforms = self
            .transforms
            .write()
            .map_err(|e| SynthError::generation(format!("Failed to acquire write lock: {}", e)))?;
        if transforms.contains_key(&name) {
            return Err(SynthError::generation(format!(
                "Transform plugin '{}' is already registered",
                name
            )));
        }
        transforms.insert(name, Arc::from(plugin));
        Ok(())
    }

    /// Get a generator plugin by name.
    pub fn get_generator(&self, name: &str) -> Option<Arc<dyn GeneratorPlugin>> {
        self.generators
            .read()
            .ok()
            .and_then(|g| g.get(name).cloned())
    }

    /// Get a transform plugin by name.
    pub fn get_transform(&self, name: &str) -> Option<Arc<dyn TransformPlugin>> {
        self.transforms
            .read()
            .ok()
            .and_then(|t| t.get(name).cloned())
    }

    /// List all registered plugins.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = Vec::new();

        if let Ok(generators) = self.generators.read() {
            for gen in generators.values() {
                plugins.push(PluginInfo {
                    name: gen.name().to_string(),
                    version: gen.version().to_string(),
                    description: gen.description().to_string(),
                    plugin_type: PluginType::Generator,
                });
            }
        }

        if let Ok(sinks) = self.sinks.read() {
            for sink_lock in sinks.values() {
                if let Ok(sink) = sink_lock.read() {
                    plugins.push(PluginInfo {
                        name: sink.name().to_string(),
                        version: String::new(),
                        description: String::new(),
                        plugin_type: PluginType::Sink,
                    });
                }
            }
        }

        if let Ok(transforms) = self.transforms.read() {
            for t in transforms.values() {
                plugins.push(PluginInfo {
                    name: t.name().to_string(),
                    version: String::new(),
                    description: String::new(),
                    plugin_type: PluginType::Transform,
                });
            }
        }

        plugins
    }

    /// Get the count of registered plugins.
    pub fn plugin_count(&self) -> usize {
        let g = self.generators.read().map(|g| g.len()).unwrap_or(0);
        let s = self.sinks.read().map(|s| s.len()).unwrap_or(0);
        let t = self.transforms.read().map(|t| t.len()).unwrap_or(0);
        g + s + t
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SynthError;
    use crate::traits::plugin::{GeneratedRecord, GenerationContext, SinkSummary};

    // Test generator plugin
    struct TestGenerator {
        name: String,
    }

    impl TestGenerator {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl GeneratorPlugin for TestGenerator {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "1.0.0"
        }
        fn description(&self) -> &str {
            "Test generator"
        }
        fn config_schema(&self) -> Option<serde_json::Value> {
            None
        }
        fn generate(
            &self,
            _config: &serde_json::Value,
            _context: &GenerationContext,
        ) -> Result<Vec<GeneratedRecord>, SynthError> {
            Ok(vec![GeneratedRecord::new("test")])
        }
    }

    // Test sink plugin
    struct TestSink {
        name: String,
        count: usize,
    }

    impl TestSink {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                count: 0,
            }
        }
    }

    impl SinkPlugin for TestSink {
        fn name(&self) -> &str {
            &self.name
        }
        fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), SynthError> {
            Ok(())
        }
        fn write_records(&mut self, records: &[GeneratedRecord]) -> Result<usize, SynthError> {
            self.count += records.len();
            Ok(records.len())
        }
        fn finalize(&mut self) -> Result<SinkSummary, SynthError> {
            Ok(SinkSummary::new(self.count))
        }
    }

    // Test transform plugin
    struct TestTransform;

    impl TransformPlugin for TestTransform {
        fn name(&self) -> &str {
            "test_transform"
        }
        fn transform(
            &self,
            mut records: Vec<GeneratedRecord>,
        ) -> Result<Vec<GeneratedRecord>, SynthError> {
            for record in &mut records {
                record.fields.insert(
                    "_transformed".to_string(),
                    serde_json::Value::Bool(true),
                );
            }
            Ok(records)
        }
    }

    #[test]
    fn test_register_and_retrieve_generator() {
        let registry = PluginRegistry::new();
        registry
            .register_generator(Box::new(TestGenerator::new("gen1")))
            .expect("should register");

        let gen = registry.get_generator("gen1");
        assert!(gen.is_some());
        assert_eq!(gen.as_ref().map(|g| g.name()), Some("gen1"));
    }

    #[test]
    fn test_register_duplicate_generator_rejected() {
        let registry = PluginRegistry::new();
        registry
            .register_generator(Box::new(TestGenerator::new("gen1")))
            .expect("first registration should succeed");

        let result = registry.register_generator(Box::new(TestGenerator::new("gen1")));
        assert!(result.is_err());
    }

    #[test]
    fn test_register_and_retrieve_sink() {
        let registry = PluginRegistry::new();
        registry
            .register_sink(Box::new(TestSink::new("sink1")))
            .expect("should register");

        let plugins = registry.list_plugins();
        assert!(plugins.iter().any(|p| p.name == "sink1"));
    }

    #[test]
    fn test_register_and_retrieve_transform() {
        let registry = PluginRegistry::new();
        registry
            .register_transform(Box::new(TestTransform))
            .expect("should register");

        let t = registry.get_transform("test_transform");
        assert!(t.is_some());
    }

    #[test]
    fn test_list_all_plugins() {
        let registry = PluginRegistry::new();
        registry
            .register_generator(Box::new(TestGenerator::new("gen1")))
            .expect("register gen");
        registry
            .register_sink(Box::new(TestSink::new("sink1")))
            .expect("register sink");
        registry
            .register_transform(Box::new(TestTransform))
            .expect("register transform");

        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 3);
        assert_eq!(registry.plugin_count(), 3);
    }

    #[test]
    fn test_get_nonexistent_plugin() {
        let registry = PluginRegistry::new();
        assert!(registry.get_generator("nonexistent").is_none());
        assert!(registry.get_transform("nonexistent").is_none());
    }

    #[test]
    fn test_empty_registry() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.plugin_count(), 0);
        assert!(registry.list_plugins().is_empty());
    }
}

//! YAML configuration for cohete.
//!
//! Provides declarative configuration (Architectural Invariant).

use crate::{
    thermal::ThermalPolicy,
    Result, Error,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoheteConfig {
    /// Configuration version
    #[serde(default = "default_version")]
    pub version: String,

    /// Device discovery settings
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Fleet configuration
    #[serde(default)]
    pub fleet: FleetConfig,

    /// Model deployment settings
    #[serde(default)]
    pub models: Vec<ModelConfig>,

    /// Thermal policies
    #[serde(default)]
    pub thermal: ThermalPoliciesConfig,

    /// Inference server settings
    #[serde(default)]
    pub inference: InferenceConfig,

    /// Provisioning settings
    #[serde(default)]
    pub provision: ProvisionYamlConfig,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Default for CoheteConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            discovery: DiscoveryConfig::default(),
            fleet: FleetConfig::default(),
            models: Vec::new(),
            thermal: ThermalPoliciesConfig::default(),
            inference: InferenceConfig::default(),
            provision: ProvisionYamlConfig::default(),
        }
    }
}

impl CoheteConfig {
    /// Load configuration from YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::from_yaml(&content)
    }

    /// Parse configuration from YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error if YAML is invalid.
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml)
            .map_err(|e| Error::InvalidYaml(e.to_string()))
    }

    /// Serialize to YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| Error::InvalidYaml(e.to_string()))
    }

    /// Save configuration to file.
    ///
    /// # Errors
    ///
    /// Returns an error if file cannot be written.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = self.to_yaml()?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}

/// Discovery configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryConfig {
    /// Discovery methods to use
    #[serde(default)]
    pub methods: Vec<DiscoveryMethod>,
}

/// Discovery method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryMethod {
    /// USB-C direct connection
    Usb,
    /// mDNS discovery
    Mdns,
    /// Static IP list
    Static(Vec<String>),
}

/// Fleet configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetConfig {
    /// Fleet name
    #[serde(default)]
    pub name: String,

    /// Device configurations
    #[serde(default)]
    pub devices: Vec<DeviceYamlConfig>,
}

/// Device YAML configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceYamlConfig {
    /// Device ID
    pub id: String,

    /// Connection type
    #[serde(default)]
    pub connection: String,

    /// IP address (for ethernet)
    pub ip: Option<String>,

    /// Thermal policy name
    #[serde(default = "default_thermal_policy")]
    pub thermal_policy: String,

    /// Memory budget in MB
    #[serde(default = "default_memory_budget")]
    pub memory_budget_mb: u64,
}

fn default_thermal_policy() -> String {
    "conservative".to_string()
}

fn default_memory_budget() -> u64 {
    6000
}

/// Model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name
    pub name: String,

    /// Model source (pacha:// URL)
    pub source: String,

    /// Quantization level
    #[serde(default)]
    pub quantization: Option<String>,

    /// Target devices ("all" or list)
    #[serde(default = "default_devices")]
    pub devices: String,
}

fn default_devices() -> String {
    "all".to_string()
}

/// Thermal policies configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalPoliciesConfig {
    /// Conservative policy
    #[serde(default)]
    pub conservative: ThermalPolicyYaml,

    /// Aggressive policy
    #[serde(default)]
    pub aggressive: ThermalPolicyYaml,
}

impl Default for ThermalPoliciesConfig {
    fn default() -> Self {
        Self {
            conservative: ThermalPolicyYaml {
                threshold_c: 65.0,
                cooldown_c: 55.0,
                check_interval_ms: 500,
            },
            aggressive: ThermalPolicyYaml {
                threshold_c: 75.0,
                cooldown_c: 65.0,
                check_interval_ms: 1000,
            },
        }
    }
}

/// Thermal policy YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalPolicyYaml {
    /// Threshold temperature in Celsius
    pub threshold_c: f32,
    /// Cooldown temperature in Celsius
    pub cooldown_c: f32,
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
}

impl Default for ThermalPolicyYaml {
    fn default() -> Self {
        Self {
            threshold_c: 65.0,
            cooldown_c: 55.0,
            check_interval_ms: 500,
        }
    }
}

impl From<ThermalPolicyYaml> for ThermalPolicy {
    fn from(yaml: ThermalPolicyYaml) -> Self {
        ThermalPolicy::custom(yaml.threshold_c, yaml.cooldown_c, yaml.check_interval_ms)
    }
}

/// Inference server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Maximum batch size
    #[serde(default = "default_batch_size")]
    pub max_batch_size: usize,

    /// Context length
    #[serde(default = "default_context")]
    pub context_length: usize,

    /// API compatibility mode
    #[serde(default = "default_api")]
    pub api_compatibility: String,
}

fn default_port() -> u16 {
    8080
}

fn default_batch_size() -> usize {
    4
}

fn default_context() -> usize {
    2048
}

fn default_api() -> String {
    "openai".to_string()
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            max_batch_size: default_batch_size(),
            context_length: default_context(),
            api_compatibility: default_api(),
        }
    }
}

/// Provisioning YAML configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvisionYamlConfig {
    /// NVMe settings
    #[serde(default)]
    pub nvme: NvmeYamlConfig,

    /// SSH settings
    #[serde(default)]
    pub ssh: SshYamlConfig,

    /// Packages to install
    #[serde(default)]
    pub packages: Vec<String>,
}

/// NVMe YAML configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeYamlConfig {
    /// Enable NVMe setup
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Mount point
    #[serde(default = "default_mount")]
    pub mount_point: String,

    /// Swap size in GB
    #[serde(default = "default_swap")]
    pub swap_size_gb: u64,
}

fn default_true() -> bool {
    true
}

fn default_mount() -> String {
    "/mnt/nvme".to_string()
}

fn default_swap() -> u64 {
    16
}

impl Default for NvmeYamlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mount_point: default_mount(),
            swap_size_gb: default_swap(),
        }
    }
}

/// SSH YAML configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshYamlConfig {
    /// Copy SSH ID
    #[serde(default = "default_true")]
    pub copy_id: bool,

    /// Config host name
    #[serde(default = "default_host")]
    pub config_host: String,
}

fn default_host() -> String {
    "jetson".to_string()
}

impl Default for SshYamlConfig {
    fn default() -> Self {
        Self {
            copy_id: true,
            config_host: default_host(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = CoheteConfig::default();
        assert_eq!(config.version, "1.0");
    }

    #[test]
    fn test_config_yaml_roundtrip() {
        let config = CoheteConfig::default();
        let yaml = config.to_yaml().unwrap();
        let parsed = CoheteConfig::from_yaml(&yaml).unwrap();
        assert_eq!(parsed.version, config.version);
    }

    #[test]
    fn test_parse_minimal_yaml() {
        let yaml = r#"
version: "1.0"
fleet:
  name: "test-fleet"
"#;
        let config = CoheteConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.fleet.name, "test-fleet");
    }

    #[test]
    fn test_thermal_policy_conversion() {
        let yaml = ThermalPolicyYaml {
            threshold_c: 70.0,
            cooldown_c: 60.0,
            check_interval_ms: 750,
        };
        let policy: ThermalPolicy = yaml.into();
        assert_eq!(policy.threshold_c, 70.0);
    }
}

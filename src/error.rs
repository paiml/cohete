//! Error types for cohete.

use std::io;
use thiserror::Error;

/// Cohete error type.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Connection failed
    #[error("Connection failed to {host}: {reason}")]
    ConnectionFailed {
        /// Target host
        host: String,
        /// Failure reason
        reason: String,
    },

    /// SSH error
    #[error("SSH error: {0}")]
    Ssh(String),

    /// USB error
    #[error("USB error: {0}")]
    Usb(String),

    /// Thermal threshold exceeded
    #[error("Thermal threshold exceeded: {current_c}°C > {threshold_c}°C")]
    ThermalExceeded {
        /// Current temperature
        current_c: f32,
        /// Threshold temperature
        threshold_c: f32,
    },

    /// Insufficient memory
    #[error("Insufficient memory: requested {requested_mb}MB, available {available_mb}MB")]
    InsufficientMemory {
        /// Requested memory in MB
        requested_mb: u64,
        /// Available memory in MB
        available_mb: u64,
    },

    /// Memory budget exceeded
    #[error("Memory budget exceeded: {used_mb}MB / {budget_mb}MB")]
    MemoryBudgetExceeded {
        /// Used memory in MB
        used_mb: u64,
        /// Budget in MB
        budget_mb: u64,
    },

    /// Power mode error
    #[error("Power mode error: {0}")]
    PowerMode(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid YAML configuration
    #[error("Invalid YAML configuration: {0}")]
    InvalidYaml(String),

    /// Quantization error
    #[error("Quantization error: {0}")]
    Quantization(String),

    /// Provisioning error
    #[error("Provisioning error: {0}")]
    Provisioning(String),

    /// Fleet error
    #[error("Fleet error: {0}")]
    Fleet(String),

    /// Parse error
    #[error("Parse error: {context}: {message}")]
    Parse {
        /// Context of the parse error
        context: String,
        /// Error message
        message: String,
    },

    /// Timeout error
    #[error("Timeout: {operation} exceeded {timeout_ms}ms")]
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout in milliseconds
        timeout_ms: u64,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Subsystem unavailable
    #[error("Subsystem {subsystem:?} unavailable: {reason}")]
    SubsystemUnavailable {
        /// The unavailable subsystem
        subsystem: Subsystem,
        /// Reason for unavailability
        reason: String,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Hardware subsystem identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Subsystem {
    /// tegrastats monitoring
    TegraStats,
    /// nvpmodel power management
    NvpModel,
    /// jetson_clocks frequency control
    JetsonClocks,
    /// NVMe storage
    Nvme,
    /// USB CDC serial
    UsbCdc,
    /// SSH connectivity
    Ssh,
    /// CUDA runtime
    Cuda,
    /// Thermal sensors
    Thermal,
    /// Power sensors
    Power,
}

impl std::fmt::Display for Subsystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TegraStats => write!(f, "tegrastats"),
            Self::NvpModel => write!(f, "nvpmodel"),
            Self::JetsonClocks => write!(f, "jetson_clocks"),
            Self::Nvme => write!(f, "NVMe"),
            Self::UsbCdc => write!(f, "USB CDC"),
            Self::Ssh => write!(f, "SSH"),
            Self::Cuda => write!(f, "CUDA"),
            Self::Thermal => write!(f, "Thermal"),
            Self::Power => write!(f, "Power"),
        }
    }
}

/// Result type alias for cohete operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_device_not_found() {
        let err = Error::DeviceNotFound("jetson-01".to_string());
        assert!(err.to_string().contains("jetson-01"));
    }

    #[test]
    fn test_error_connection_failed() {
        let err = Error::ConnectionFailed {
            host: "192.168.55.1".to_string(),
            reason: "timeout".to_string(),
        };
        assert!(err.to_string().contains("192.168.55.1"));
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_error_ssh() {
        let err = Error::Ssh("authentication failed".to_string());
        assert!(err.to_string().contains("authentication"));
    }

    #[test]
    fn test_error_usb() {
        let err = Error::Usb("device busy".to_string());
        assert!(err.to_string().contains("device busy"));
    }

    #[test]
    fn test_error_thermal_exceeded() {
        let err = Error::ThermalExceeded {
            current_c: 75.0,
            threshold_c: 70.0,
        };
        assert!(err.to_string().contains("75"));
        assert!(err.to_string().contains("70"));
    }

    #[test]
    fn test_error_insufficient_memory() {
        let err = Error::InsufficientMemory {
            requested_mb: 6000,
            available_mb: 4000,
        };
        assert!(err.to_string().contains("6000"));
        assert!(err.to_string().contains("4000"));
    }

    #[test]
    fn test_error_memory_budget_exceeded() {
        let err = Error::MemoryBudgetExceeded {
            used_mb: 5000,
            budget_mb: 4000,
        };
        assert!(err.to_string().contains("5000"));
        assert!(err.to_string().contains("4000"));
    }

    #[test]
    fn test_error_power_mode() {
        let err = Error::PowerMode("invalid mode".to_string());
        assert!(err.to_string().contains("invalid mode"));
    }

    #[test]
    fn test_error_storage() {
        let err = Error::Storage("disk full".to_string());
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_error_config() {
        let err = Error::Config("missing field".to_string());
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_error_invalid_yaml() {
        let err = Error::InvalidYaml("syntax error".to_string());
        assert!(err.to_string().contains("syntax error"));
    }

    #[test]
    fn test_error_quantization() {
        let err = Error::Quantization("unsupported format".to_string());
        assert!(err.to_string().contains("unsupported format"));
    }

    #[test]
    fn test_error_provisioning() {
        let err = Error::Provisioning("failed to setup".to_string());
        assert!(err.to_string().contains("failed to setup"));
    }

    #[test]
    fn test_error_fleet() {
        let err = Error::Fleet("no devices available".to_string());
        assert!(err.to_string().contains("no devices"));
    }

    #[test]
    fn test_error_parse() {
        let err = Error::Parse {
            context: "tegrastats".to_string(),
            message: "invalid format".to_string(),
        };
        assert!(err.to_string().contains("tegrastats"));
        assert!(err.to_string().contains("invalid format"));
    }

    #[test]
    fn test_error_timeout() {
        let err = Error::Timeout {
            operation: "connection".to_string(),
            timeout_ms: 5000,
        };
        assert!(err.to_string().contains("connection"));
        assert!(err.to_string().contains("5000"));
    }

    #[test]
    fn test_error_subsystem_unavailable() {
        let err = Error::SubsystemUnavailable {
            subsystem: Subsystem::TegraStats,
            reason: "not installed".to_string(),
        };
        assert!(err.to_string().contains("TegraStats"));
        assert!(err.to_string().contains("not installed"));
    }

    #[test]
    fn test_error_internal() {
        let err = Error::Internal("unexpected state".to_string());
        assert!(err.to_string().contains("unexpected state"));
    }

    #[test]
    fn test_error_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err: Error = io_err.into();
        assert!(err.to_string().contains("access denied"));
    }

    #[test]
    fn test_subsystem_display() {
        assert_eq!(Subsystem::TegraStats.to_string(), "tegrastats");
        assert_eq!(Subsystem::NvpModel.to_string(), "nvpmodel");
        assert_eq!(Subsystem::JetsonClocks.to_string(), "jetson_clocks");
        assert_eq!(Subsystem::Nvme.to_string(), "NVMe");
        assert_eq!(Subsystem::UsbCdc.to_string(), "USB CDC");
        assert_eq!(Subsystem::Ssh.to_string(), "SSH");
        assert_eq!(Subsystem::Cuda.to_string(), "CUDA");
        assert_eq!(Subsystem::Thermal.to_string(), "Thermal");
        assert_eq!(Subsystem::Power.to_string(), "Power");
    }

    #[test]
    fn test_subsystem_equality() {
        assert_eq!(Subsystem::TegraStats, Subsystem::TegraStats);
        assert_ne!(Subsystem::TegraStats, Subsystem::NvpModel);
    }

    #[test]
    fn test_subsystem_clone() {
        let sub = Subsystem::Cuda;
        let cloned = sub;
        assert_eq!(sub, cloned);
    }

    #[test]
    fn test_subsystem_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Subsystem::TegraStats);
        set.insert(Subsystem::TegraStats);
        assert_eq!(set.len(), 1);
        set.insert(Subsystem::Cuda);
        assert_eq!(set.len(), 2);
    }
}

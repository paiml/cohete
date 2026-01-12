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
    fn test_error_display() {
        let err = Error::ThermalExceeded {
            current_c: 75.0,
            threshold_c: 70.0,
        };
        assert!(err.to_string().contains("75"));
        assert!(err.to_string().contains("70"));
    }

    #[test]
    fn test_subsystem_display() {
        assert_eq!(Subsystem::TegraStats.to_string(), "tegrastats");
        assert_eq!(Subsystem::NvpModel.to_string(), "nvpmodel");
    }

    #[test]
    fn test_memory_error() {
        let err = Error::InsufficientMemory {
            requested_mb: 6000,
            available_mb: 4000,
        };
        assert!(err.to_string().contains("6000"));
        assert!(err.to_string().contains("4000"));
    }
}

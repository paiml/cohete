//! # Cohete: NVIDIA Jetson Edge Hardware Integration
//!
//! Cohete (Spanish: "rocket") provides safe, pure Rust interfaces to NVIDIA Jetson
//! edge hardware for the Sovereign AI Stack.
//!
//! ## Features
//!
//! - **Device Discovery**: USB CDC, mDNS, and SSH-based Jetson detection
//! - **Thermal Monitoring**: tegrastats integration with circuit breakers
//! - **Memory Management**: Budget-aware allocation for constrained devices
//! - **Power Management**: nvpmodel and jetson_clocks control
//! - **Fleet Orchestration**: Multi-device management with repartir integration
//! - **Model Quantization**: Memory-aware quantization for edge deployment
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use cohete::{device::JetsonDevice, thermal::TegraMonitor};
//!
//! #[tokio::main]
//! async fn main() -> cohete::Result<()> {
//!     // Discover Jetson devices
//!     let devices = JetsonDevice::discover_all().await?;
//!
//!     for device in devices {
//!         let mut monitor = TegraMonitor::connect(&device)?;
//!         let stats = monitor.sample()?;
//!         println!("{}: GPU {}°C, Memory {}MB free",
//!             device.id(),
//!             stats.gpu_temp,
//!             stats.available_memory_mb
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Integration with Sovereign AI Stack
//!
//! Cohete integrates with other stack components:
//!
//! - **trueno**: ARM NEON SIMD backend selection
//! - **repartir**: Remote executor for distributed inference
//! - **pacha**: Secure model deployment with Ed25519 signatures
//! - **realizar**: On-device inference server
//! - **renacer**: Thermal and performance tracing
//!
//! ## Declarative Configuration
//!
//! Full functionality via YAML (Architectural Invariant):
//!
//! ```yaml
//! # cohete.yaml
//! fleet:
//!   devices:
//!     - id: jetson-01
//!       connection: usb
//!       thermal_policy: conservative
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod device;
pub mod thermal;
pub mod memory;
pub mod power;
pub mod storage;
pub mod fleet;
pub mod quantize;
pub mod provision;
pub mod config;
mod error;

// Re-exports
pub use error::{Error, Result, Subsystem};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported Jetson models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JetsonModel {
    /// Jetson Orin Nano 4GB
    OrinNano4GB,
    /// Jetson Orin Nano 8GB
    OrinNano8GB,
    /// Jetson Orin NX 8GB
    OrinNX8GB,
    /// Jetson Orin NX 16GB
    OrinNX16GB,
    /// Jetson AGX Orin 32GB
    AgxOrin32GB,
    /// Jetson AGX Orin 64GB
    AgxOrin64GB,
    /// Unknown Jetson model
    Unknown,
}

impl JetsonModel {
    /// Total memory in MB
    #[must_use]
    pub const fn memory_mb(&self) -> u64 {
        match self {
            Self::OrinNano4GB => 4096,
            Self::OrinNano8GB => 8192,
            Self::OrinNX8GB => 8192,
            Self::OrinNX16GB => 16384,
            Self::AgxOrin32GB => 32768,
            Self::AgxOrin64GB => 65536,
            Self::Unknown => 0,
        }
    }

    /// CUDA cores
    #[must_use]
    pub const fn cuda_cores(&self) -> u32 {
        match self {
            Self::OrinNano4GB => 512,
            Self::OrinNano8GB => 1024,
            Self::OrinNX8GB => 1024,
            Self::OrinNX16GB => 1024,
            Self::AgxOrin32GB => 2048,
            Self::AgxOrin64GB => 2048,
            Self::Unknown => 0,
        }
    }

    /// AI performance in TOPS
    #[must_use]
    pub const fn tops(&self) -> u32 {
        match self {
            Self::OrinNano4GB => 20,
            Self::OrinNano8GB => 40,
            Self::OrinNX8GB => 70,
            Self::OrinNX16GB => 100,
            Self::AgxOrin32GB => 200,
            Self::AgxOrin64GB => 275,
            Self::Unknown => 0,
        }
    }
}

impl std::fmt::Display for JetsonModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrinNano4GB => write!(f, "Jetson Orin Nano 4GB"),
            Self::OrinNano8GB => write!(f, "Jetson Orin Nano 8GB"),
            Self::OrinNX8GB => write!(f, "Jetson Orin NX 8GB"),
            Self::OrinNX16GB => write!(f, "Jetson Orin NX 16GB"),
            Self::AgxOrin32GB => write!(f, "Jetson AGX Orin 32GB"),
            Self::AgxOrin64GB => write!(f, "Jetson AGX Orin 64GB"),
            Self::Unknown => write!(f, "Unknown Jetson"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jetson_model_memory() {
        assert_eq!(JetsonModel::OrinNano8GB.memory_mb(), 8192);
        assert_eq!(JetsonModel::AgxOrin64GB.memory_mb(), 65536);
    }

    #[test]
    fn test_jetson_model_display() {
        assert_eq!(
            JetsonModel::OrinNano8GB.to_string(),
            "Jetson Orin Nano 8GB"
        );
    }

    #[test]
    fn test_jetson_model_tops() {
        assert_eq!(JetsonModel::OrinNano8GB.tops(), 40);
        assert_eq!(JetsonModel::AgxOrin64GB.tops(), 275);
    }
}

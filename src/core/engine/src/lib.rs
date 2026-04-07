//! mstore-core: zero-kernel user-space storage engine.
//!
//! This crate provides the fundamental abstractions for the m-store engine:
//! - I/O path over SPDK lock-free NVMe submission queues
//! - Plugin trait for domain-specific storage backends
//! - Monadic composition of storage operations

/// Trait implemented by every m-store backend plugin.
pub trait Plugin: Send + Sync {
    /// Human-readable name of the plugin (e.g. "nvme", "rdma").
    fn name(&self) -> &'static str;

    /// Initialize the plugin with the given configuration blob.
    fn init(&self, config: &[u8]) -> Result<(), PluginError>;
}

/// Errors produced by the storage engine and its plugins.
#[derive(Debug)]
pub enum PluginError {
    InitFailed(String),
    IoError(String),
    Unsupported(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::InitFailed(msg) => write!(f, "plugin init failed: {msg}"),
            PluginError::IoError(msg) => write!(f, "I/O error: {msg}"),
            PluginError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

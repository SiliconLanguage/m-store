//! mstore-plugin-rdma: zero-copy RDMA transport for m-store.
//!
//! This plugin binds to an RDMA-capable NIC (via ibverbs / rdma-core) and
//! provides a zero-copy remote DMA transport as an m-store [`Plugin`] backend.

use mstore_core::{Plugin, PluginError};

/// RDMA plugin for zero-copy remote DMA transfers.
pub struct RdmaPlugin;

impl Plugin for RdmaPlugin {
    fn name(&self) -> &'static str {
        "rdma"
    }

    fn init(&self, _config: &[u8]) -> Result<(), PluginError> {
        // TODO: open ibverbs context and allocate protection domain.
        Ok(())
    }
}

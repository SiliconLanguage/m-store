//! mstore-plugin-nvme: SPDK NVMe queue-pair bindings for m-store.
//!
//! This plugin attaches to an SPDK NVMe controller and exposes its lock-free
//! submission/completion queue pairs as an m-store [`Plugin`] backend.

use mstore_core::{Plugin, PluginError};

/// NVMe plugin backed by SPDK queue pairs.
pub struct NvmePlugin;

impl Plugin for NvmePlugin {
    fn name(&self) -> &'static str {
        "nvme"
    }

    fn init(&self, _config: &[u8]) -> Result<(), PluginError> {
        // TODO: call spdk_nvme_probe() and attach controllers.
        Ok(())
    }
}

# m-store

**Monadic Store: Hardware-Software Co-Design for Zero-Kernel User-Space Storage Systems.**

m-store is a zero-kernel, user-space storage engine optimised for hyperscale AI workloads.
It pairs a plugin-based Rust core with an SPDK NVMe back-end and an MLIR/eBPF compiler
pipeline to deliver deterministic, ultra-low-latency I/O.

## Architecture

```
m-store/
├── core/                     # Rust storage engine (Plugin trait, I/O path)
├── plugins/
│   ├── nvme/                 # SPDK NVMe queue-pair plugin
│   └── rdma/                 # ibverbs zero-copy RDMA plugin
├── compiler/
│   ├── mlir/dialects/        # MLIR dialect for storage ops (mstore.mlir)
│   └── ebpf/hooks/           # eBPF I/O latency hooks (io_hook.bpf.c)
├── vendor/spdk/              # SPDK submodule (lock-free NVMe queues)
└── docs/                     # Architecture decision records
```

## Prerequisites

| Tool | Minimum version |
|------|----------------|
| Rust / Cargo | 1.78 |
| LLVM / MLIR | 18 |
| libbpf | 1.3 |
| SPDK | see `vendor/spdk` submodule |

## Quick Start

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/SiliconLanguage/m-store.git
cd m-store

# Build the Rust workspace
cargo build

# Bootstrap SPDK (run once)
cd vendor/spdk && ./scripts/pkgdep.sh && ./configure && make -j$(nproc)
```

## License

BSD-2-Clause-Patent — see [LICENSE](LICENSE).
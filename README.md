# 🗄️ m-store: Monadic Store Architecture
**A Multi-Modal, Zero-Kernel Data Platform for Hyperscale AI**

## 📖 The Vision
The physical constraints governing high-performance AI infrastructure have shifted irrevocably from the storage medium to the operating system's kernel stack. Modern NVMe SSDs deliver microsecond latencies, meaning the legacy Linux Virtual File System (VFS), POSIX compliance layers, and hardware interrupt handlers now account for a massive percentage of overall I/O latency. 

**`m-store`** is a zero-kernel, zero-copy storage engine built entirely in user space. Applying the "Monadic Paradigm," it treats the host CPU as a pure computational engine, isolating all state mutation and storage side-effects to out-of-band hardware. 

Rather than forcing a rigid POSIX filesystem onto every workload, `m-store` operates as a foundational **Key-Array-Value** substrate. It exposes its zero-copy data plane through highly specialized, dynamically compiled plugins tailored for the exact bottlenecks of modern LLM inference and training.

---

## 🏗️ 1. The Core Substrate: Zero-Copy Data Plane
`m-store` abandons traditional blocking I/O in favor of an asynchronous, hardware-mapped data plane:
*   **SPDK Blobstore & Thread-per-Core Execution:** `m-store` maps application logical structures directly to physical NVMe space via the Storage Performance Development Kit (SPDK) Blobstore. By polling NVMe hardware completion queues directly from user-level threads, it completely eradicates hardware interrupts and context switches.
*   **Iov/Ior Native Client Architecture:** Inspired by DeepSeek's 3FS `USRBIO` interface, `m-store` utilizes an asynchronous, `io_uring`-style API. Applications queue commands in a shared memory ring (**Ior**), while bulk data moves directly between the NIC and a registered memory vector (**Iov**) via RDMA, achieving true zero-copy transfers.

---

## 🧩 2. Domain-Specific AI Plugins
To saturate 400 Gbps network fabrics and PCIe Gen 5 buses, `m-store` routes specialized workloads through purpose-built plugins:

### 🔌 Plugin A: LLM KV Cache Store (Radix & PagedAttention)
*   **The Bottleneck:** LLM inference is severely bounded by GPU VRAM limits. Evicting and recomputing the KV cache for inactive users takes seconds.
*   **The Architecture:** This plugin natively understands PagedAttention block tables and SGLang's RadixAttention prefix trees. By bypassing the CPU "bounce buffer," `m-store` can swap a 4GB KV cache from NVMe back to GPU VRAM via PCIe in just 200–400ms.

### 🔌 Plugin B: Vector & Graph Store (RAG)
*   **The Bottleneck:** Billion-scale Approximate Nearest Neighbor (ANN) graphs (like DiskANN) require massive volumes of 8KB random reads, which causes catastrophic page cache thrashing and read amplification in the Linux VFS.
*   **The Architecture:** `m-store` avoids the page cache entirely. It dynamically loads graph routing lists into device-side caches, returning only the exact requested vector payload to the host, enabling sub-millisecond retrieval latencies for RAG pipelines.

### 🔌 Plugin C: Tensor Array & MoE Weight Store
*   **The Bottleneck:** Swapping massive expert parameters for Mixture of Experts (MoE) models or writing synchronous checkpoints stalls training clusters.
*   **The Architecture:** Modeled after 3FS (which achieves up to 6.6 TiB/s read bandwidth), this plugin streams expert weights directly from distributed storage into GPU memory via RDMA, making checkpointing an asynchronous, lock-free operation.

---

## ⚡ 3. The Compiler & Execution Engine (MLIR + eBPF)
Allowing arbitrary code inside a storage engine is a security risk, but legacy SQL is too rigid for tensor mathematics. `m-store` implements a secure, **two-tiered execution model**:

1.  **Frontend (MLIR Translation):** Users write declarative Python DSLs (similar to Triton or SGLang). The `m-store` compiler translates this Abstract Syntax Tree (AST) into a custom **MLIR (Multi-Level Intermediate Representation)** dialect, applying automatic memory coalescing and hardware-specific optimizations before lowering it to physical instructions.
2.  **Backend (eBPF & Near-Storage Compute):** 
    *   **In-Kernel XRP Hooks:** For operations that must execute on the host, `m-store` compiles the MLIR into eBPF bytecode. Utilizing the **XRP (eXpress Resubmission Path)** framework, this eBPF code runs safely inside the NVMe driver’s interrupt handler. It can chase pointers through a B-tree or evaluate vectors and immediately resubmit the next I/O request without ever waking up the user-space process—improving throughput by up to 2.5x.
    *   **DPU/CSD Delegation:** Where available, `m-store` ships execution directly to out-of-band hardware. We utilize dual-layer compression on Computational Storage Drives (like Alibaba's PolarCSD) and offload AES-XTS encryption to the ARM cores of NVIDIA BlueField DPUs using the DOCA SNAP framework.

---
## 📜 License
`m-store` is released under the **BSD-2-Clause Plus Patent License**. This ensures explicit patent protection for our hardware-software co-design while maintaining full legal compatibility with GPLv2 for seamless integration into enterprise Linux environments.

*Built for the post-Von Neumann era. The host CPU is a pure computational engine; all state, network, and storage side-effects are isolated to out-of-band hardware.*

Directory Structure
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

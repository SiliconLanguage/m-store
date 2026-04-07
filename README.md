# 🗄️ m-store: Monadic Store Architecture
**A Multi-Modal, Zero-Kernel Data Platform for Hyperscale AI**

## 📖 The Vision
The physical constraints governing high-performance AI infrastructure have shifted irrevocably from the storage medium to the operating system's kernel stack. Modern NVMe SSDs deliver microsecond latencies, meaning the legacy Linux Virtual File System (VFS), POSIX compliance layers, and hardware interrupt handlers now account for a massive percentage of overall I/O latency. 

**`m-store`** is a zero-kernel, zero-copy storage engine built entirely in user space. Applying the "Monadic Paradigm," it treats the host CPU as a pure computational engine, isolating all state mutation and storage side-effects to out-of-band hardware. 

Rather than forcing a rigid POSIX filesystem onto every workload, `m-store` operates as a foundational **Key-Array-Value** substrate. It exposes its zero-copy data plane through highly specialized, dynamically compiled plugins tailored for the exact bottlenecks of modern LLM inference, training, and autonomous agentic workflows.

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
*   **The Bottleneck:** Swapping massive expert parameters for Mixture of Experts (MoE) models or writing synchronous checkpoints stalls training clusters. Fetching required experts via standard kernel I/O incurs latency spikes of ~10 milliseconds, crippling token generation.
*   **The Architecture:** Modeled after 3FS (which achieves up to 6.6 TiB/s read bandwidth), this plugin streams expert weights directly from distributed storage into GPU memory via user-space DMA. This entirely bypasses the host CPU bounce buffers and eliminates PCIe bottleneck stalls, preserving interactive token generation latencies without sacrificing model accuracy.

### 🔌 Plugin D: Agentic Episodic Memory (Engram Cache)
*   **The Bottleneck:** As AI agents evolve into autonomous entities, they require persistent, sub-millisecond retrieval of specific past interactions (episodic memory). Forcing agents to store this state as unstructured markdown "Wikis" in a slow POSIX filesystem breaks the zero-kernel paradigm and hinders scalability.
*   **The Architecture:** `m-store` natively manages the agentic memory spectrum by supporting **Engrams**—immutable, cryptographically verifiable traces of an agent's past reasoning chains and tool actions. By routing these event logs into high-speed `Ior` queues and vectorizing them on the fly, agents can rapidly perform semantic searches over an append-only log without overflowing their active LLM context window.

---

## ⚡ 3. The Compiler & Execution Engine (MLIR + eBPF)
Allowing arbitrary code inside a storage engine is a security risk, but legacy SQL is too rigid for tensor mathematics. `m-store` implements a secure, **two-tiered execution model**:

*   **Tier 1: In-Kernel / Near-Storage Compute (The XRP Pattern):** Leveraging frameworks like the eXpress Resubmission Path (XRP), `m-store` injects verified extended Berkeley Packet Filter (eBPF) functions directly into the NVMe driver's hardware interrupt completion handler. This permits the storage device to autonomously traverse complex B-trees, filter vectors, and chain physical logical block address (LBA) read requests without ever triggering a context switch or copying intermediate blocks to user space.
*   **Tier 2: The MLIR / LLVM Compiler Continuum:** To support highly specialized AI data manipulations, `m-store` utilizes MLIR to translate high-level mathematical intent (e.g., Triton or StableHLO dialects) directly into optimized hardware execution paths. Driven by the overarching **Tensorplane** agentic control plane, the system can autonomously generate, sandbox, and atomically hot-swap C++ or WebAssembly storage kernels to adapt to specific workload bottlenecks at machine speed.

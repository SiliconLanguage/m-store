# 🗄️ m-store: Monadic Store Architecture
**A Multi-Modal, Zero-Kernel Data Platform for Hyperscale AI**

> 📖 **Read the Manifesto:** Discover the overarching "Monadic Taxonomy," our hardware-software co-design philosophy, and how `m-store` acts as the physical data plane for autonomous AI agents in the [SiliconLanguage Vision Document](https://github.com/SiliconLanguage/.github/blob/main/docs/vision.md).

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

## 🧩 2. Memory & State Orchestration Fabric
The traditional von Neumann architecture has become the primary bottleneck for generative AI. As model parameters scale and context windows expand to millions of tokens, the "memory wall" forces a convergence where hardware and software must be co-designed. `m-store` addresses this by acting as a unified, disaggregated memory fabric, directly integrating with the industry's leading inference engines and frameworks:

### 🔌 vLLM & PagedAttention Substrate (`src/fabric/plugins/vllm_paged_kv.cpp`)
*   **The Architecture:** The primary bottleneck in LLM serving is KV cache memory waste. `m-store` natively supports vLLM’s **PagedAttention**, which partitions the KV cache into non-contiguous, fixed-size pages (similar to OS virtual memory), increasing memory utilization by 2-4x and enabling much larger batch sizes. By bypassing the CPU "bounce buffer," it can swap a 4GB KV cache from NVMe back to GPU VRAM via PCIe in just 200–400ms.
*   **Triton Backend Alignment:** Aligning with vLLM's transition to a Triton-based attention backend, `m-store` exposes its memory vectors directly to persistent kernels that pull metadata directly from GPU memory, completely bypassing the overhead of re-launching kernels for every request in a batch.

### 🔌 SGLang & Persistent Radix Trees (`src/fabric/plugins/sglang_radix.cpp`)
*   **The Architecture:** For multi-step reasoning and complex prompt sharing, `m-store` integrates with SGLang's **RadixAttention**. It treats the KV cache as a persistent prefix tree.
*   **Zero-Copy Memoization:** If multiple requests share a common prefix (e.g., a system prompt), `m-store` caches the KV tensors for that prefix and reuses them, reducing the inference "prefill" computation from $O(L_{total})$ to $O(L_{new})$. It utilizes a Cache-Aware Scheduler to prioritize requests that match data already resident in GPU HBM, minimizing cache thrashing.

### 🔌 AI-Optimized Distributed Storage & CXL Pooling
*   **Disaggregated Object Store:** Inspired by architectures like DeepSeek 3FS and Intel DAOS, `m-store` replaces traditional filesystems with a disaggregated architecture where thousands of NVMe SSDs are pooled into a single namespace, prioritizing massive random-read throughput.
*   **CXL 3.1 Dynamic NUMA:** To eliminate the boundaries of rigid, server-local memory hierarchies, `m-store` is designed to support Compute Express Link (CXL) 3.1. By leveraging **Global Integrated Memory (GIM)** and **Port-Based Routing (PBR)**, compute nodes and memory devices connect through CXL switches. This creates a "dynamic NUMA" environment where memory is hot-plugged into a server's address space based on workload demand, eliminating overprovisioning and reducing Total Cost of Ownership (TCO) by up to 25%.

### 🔌 Vector & Graph Store Substrate (RAG)
*   **The Bottleneck:** Billion-scale Approximate Nearest Neighbor (ANN) graphs (like DiskANN) require massive volumes of 8KB random reads, which causes catastrophic page cache thrashing and read amplification in the Linux VFS.
*   **The Architecture:** `m-store` avoids the page cache entirely. It dynamically loads graph routing lists into device-side caches, returning only the exact requested vector payload to the host, enabling sub-millisecond retrieval latencies for RAG pipelines.

### 🔌 Tensor Array & MoE Weight Substrate
*   **The Bottleneck:** Swapping massive expert parameters for Mixture of Experts (MoE) models or writing synchronous checkpoints stalls training clusters. Fetching required experts via standard kernel I/O incurs latency spikes of ~10 milliseconds, crippling token generation.
*   **The Architecture:** Modeled after 3FS (which achieves up to 6.6 TiB/s read bandwidth), this plugin streams expert weights directly from distributed storage into GPU memory via user-space DMA. This entirely bypasses the host CPU bounce buffers and eliminates PCIe bottleneck stalls, preserving interactive token generation latencies without sacrificing model accuracy.

### 🔌 MCP Agentic Episodic Memory (`src/fabric/plugins/mcp_engrams.cpp`)
*   **The Bottleneck:** As AI agents evolve into autonomous entities, they require persistent, sub-millisecond retrieval of specific past interactions (episodic memory). Forcing agents to store this state as unstructured markdown "Wikis" in a slow POSIX filesystem breaks the zero-kernel paradigm and hinders scalability.
*   **The Architecture:** `m-store` natively manages the agentic memory spectrum by supporting **Engrams**—immutable, cryptographically verifiable traces of an agent's past reasoning chains and tool actions. By routing these event logs into high-speed `Ior` queues and vectorizing them on the fly, agents can rapidly perform semantic searches over an append-only log without overflowing their active LLM context window.

---

## ⚡ 3. The Compiler & Execution Engine (MLIR + eBPF)
Allowing arbitrary code inside a storage engine is a security risk, but legacy SQL is too rigid for tensor mathematics. `m-store` implements a secure, **two-tiered execution model**:

*   **Tier 1: In-Kernel / Near-Storage Compute (The XRP Pattern):** Leveraging frameworks like the eXpress Resubmission Path (XRP), `m-store` injects verified extended Berkeley Packet Filter (eBPF) functions directly into the NVMe driver's hardware interrupt completion handler. This permits the storage device to autonomously traverse complex B-trees, filter vectors, and chain physical logical block address (LBA) read requests without ever triggering a context switch or copying intermediate blocks to user space.
*   **Tier 2: The MLIR / LLVM Compiler Continuum:** To support highly specialized AI data manipulations, `m-store` utilizes MLIR to translate high-level mathematical intent (e.g., Triton or StableHLO dialects) directly into optimized hardware execution paths. Driven by the overarching **Tensorplane** agentic control plane, the system can autonomously generate, sandbox, and atomically hot-swap C++ or WebAssembly storage kernels to adapt to specific workload bottlenecks at machine speed.

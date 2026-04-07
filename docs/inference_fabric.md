# 🗄️ m-store: The Unified Memory & State Orchestration Fabric
**Architectural Deep-Dive: Natively Integrating vLLM, SGLang, and CXL 3.1**

## 📖 Executive Summary
The transition of Large Language Models (LLMs) from stateless text generators to stateful, multi-agent reasoning engines has fundamentally broken the traditional von Neumann storage hierarchy. As context windows scale to millions of tokens, the "memory wall" dictates that inference performance is no longer compute-bound, but entirely bounded by memory bandwidth and Key-Value (KV) cache management. 

This document details the architecture of the **Memory & State Orchestration Fabric** residing within the `src/fabric/` layer of `m-store`. By providing a zero-kernel, RDMA-first data plane, `m-store` natively integrates with the industry's leading inference frameworks (vLLM and SGLang) and emerging hardware topologies (CXL 3.1). It transforms disparate NVMe drives and remote memory into a singular, dynamic NUMA space directly addressable by GPU Tensor Cores.

---

## 🏗️ Disaggregated KV Substrate for vLLM & PagedAttention
vLLM revolutionized inference serving with **PagedAttention**, which treats the KV cache like an operating system's virtual memory, dividing it into non-contiguous, fixed-size blocks to eliminate fragmentation and increase batch sizes. However, as models scale, the industry is shifting to **Disaggregated Prefill/Decode (P/D)** architectures to isolate the compute-heavy prefill phase from the memory-bandwidth-bound decode phase across different GPU clusters.

### The Zero-Copy Connector Integration
In a disaggregated setup, prefill workers generate massive KV caches that must be transferred to decode workers. Relying on traditional TCP/IP or kernel-mediated storage connectors incurs severe CPU overhead and latency spikes. 

**`m-store` operates as the native Shared KV-Cache Service** for vLLM. By leveraging **GPUDirect Storage (GDS)** and zero-copy Direct Memory Access (DMA), `m-store` creates a frictionless pipeline:
*   Prefill nodes use user-space `Ior`/`Iov` queues to write KV pages directly from GPU VRAM to the NVMe-oF data plane, bypassing the CPU bounce buffer entirely.
*   Decode nodes instantly stream these KV tensors back into their VRAM via PCIe, saturating network links at bare-metal speeds.

### Triton Backend Alignment & Persistent Kernels
To ensure performance portability across NVIDIA and AMD silicon, vLLM utilizes a Triton-based attention backend. A significant challenge with Triton in dynamic inference environments is the software launch overhead when re-evaluating launch grids for variable batch sizes, which traditionally breaks CUDA/HIP graph compatibility.

`m-store`’s memory vectors (`Iov`) are designed to feed directly into vLLM’s **persistent Triton kernels**. By utilizing a static launch grid strategy with dynamic work assignment, the execution graph remains frozen, completely eliminating Triton Just-In-Time (JIT) recompilation and launch overheads. This allows `m-store` to stream KV data into the matrix multiplications without interrupting the execution wave.

---

## 🌳 Persistent Radix Trees & Zero-Overhead Eviction for SGLang
While vLLM optimizes independent requests, **SGLang** optimizes for **computational reuse** in complex, agentic workflows (e.g., Tree of Thoughts (ToT) or few-shot learning). SGLang achieves this via **RadixAttention**, treating the KV cache as a persistent, content-addressable prefix tree. 

### Mitigating "Memory Amnesia"
If multiple requests share a common system prompt or conversation history, RadixAttention memoizes the prefix. This results in massive throughput increases in highly structured workloads. However, retaining infinite KV caches is impossible due to strict GPU VRAM limits, necessitating complex Least Recently Used (LRU) eviction policies.

### `m-store` Radix Tree Interception
When SGLang identifies leaf nodes with zero reference counts for eviction, `m-store`'s `sglang_radix.cpp` plugin intercepts the operation. 
*   Instead of permanently deleting the tensors or offloading them to slow CPU RAM via the Linux Virtual File System (VFS), `m-store` persists the exact Radix Tree structure to its high-speed NVMe-oF arrays.
*   When a suspended agentic workflow resumes, `m-store` executes a rapid retrieval of the exact prefix tree state, swapping it directly back into VRAM. 
*   This perfectly preserves SGLang's multi-turn throughput advantages without exhausting physical VRAM capacity or stalling the GPU.

---

## 🌐 CXL 3.1 Global Integrated Memory (GIM) & Dynamic NUMA
As model sizes explode, relying strictly on server-local memory hierarchies leads to severe overprovisioning and stranded resources. The `m-store` architecture anticipates the deployment of **Compute Express Link (CXL) 3.1**, which transforms PCIe interconnects into fully cache-coherent, rack-level memory fabrics.

### The Tiered Pooling Strategy
`m-store` models the physical hardware as a bipartite graph of compute hosts and Multi-Headed Devices (MHDs) connected via CXL switches. It orchestrates a sophisticated two-tier disaggregated memory approach:

*   **Tier 1: CXL Cache-Coherent Pools (Global Integrated Memory):** Utilizing CXL 3.1 Port-Based Routing (PBR) and hardware-managed MESI coherence, `m-store` maps remote CXL memory expanders directly into the host's physical address space. This creates a **"Dynamic NUMA"** environment with ultra-low latency, ideal for hot KV cache storage and active Model Context Protocol (MCP) tool execution.
*   **Tier 2: NVMe-oF Bulk Storage:** For colder episodic memories, dormant Radix Tree branches, and heavy MoE expert weights, data is tiered down to the lock-free SPDK blobstores.

By decoupling the memory capacity from the compute nodes, `m-store` allows inference engines to access terabytes of shared state dynamically. This eliminates redundant data transfers across the cluster, improves memory utilization, and reduces the Total Cost of Ownership (TCO) of the AI factory.

---

## 🔬 Conclusion
The `m-store` Memory & State Orchestration Fabric erases the boundaries between local RAM, GPU VRAM, and remote storage. By providing a hardware-accurate, zero-copy data path tailored explicitly for vLLM's PagedAttention, SGLang's Radix Trees, and CXL 3.1 fabrics, `m-store` ensures that the multi-million dollar GPUs driving the modern AI factory are never starved for data.
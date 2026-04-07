cmake_minimum_required(VERSION 3.14)
project(m_store)

set(CMAKE_CXX_STANDARD 17)

# Memory and State Orchestration Fabric Architecture
# Include paths for src/core/ and src/fabric/
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src/core)
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src/fabric)

# Add Fabric plugins
add_library(mstore_fabric_plugins STATIC
    src/fabric/plugins/vllm_paged_kv.cpp
    src/fabric/plugins/sglang_radix.cpp
    src/fabric/plugins/mcp_engrams.cpp
)

# Note: The raw NVMe/SPDK block mapping and RDMA data plane components in 
# src/core are currently implemented in Rust and built via Cargo.
# They can be integrated with CMake using Corrosion (cmake-cargo) if needed.

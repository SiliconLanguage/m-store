// MStore MLIR Dialect — placeholder
//
// This file will define the `mstore` MLIR dialect that lowers high-level
// storage operations (read, write, alloc, free) to hardware-specific
// primitives via the SPDK NVMe and RDMA plugins.
//
// Planned ops:
//   mstore.read  %buf, %lba, %len  : (memref, i64, i64) -> ()
//   mstore.write %buf, %lba, %len  : (memref, i64, i64) -> ()
//   mstore.alloc %size             : (i64) -> memref
//   mstore.free  %buf              : (memref) -> ()

// TODO: implement dialect definition using MLIR TableGen (.td) and C++ bindings.

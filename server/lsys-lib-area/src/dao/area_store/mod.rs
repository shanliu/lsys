mod store_memory;
pub use store_memory::*;

#[cfg(feature = "index-disk")]
mod store_disk;
#[cfg(feature = "index-disk")]
pub use store_disk::*;

//公开接口
#[cfg(feature = "barcode")]
mod barcode;
#[cfg(feature = "barcode")]
pub use barcode::*;

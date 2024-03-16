mod setting;
pub use setting::*;
mod setting_site;
pub use setting_site::*;

#[cfg(feature = "docs")]
mod docs;
#[cfg(feature = "docs")]
pub use docs::*;
#[macro_use]
mod area;
pub use area::*;

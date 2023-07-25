mod hash;
pub use crate::WIDTH;

#[cfg(feature = "alloc")]
mod gadget;

#[cfg(feature = "merkle")]
pub mod merkle;
pub mod truncated;

pub use hash::hash;

#[cfg(feature = "alloc")]
pub use gadget::gadget;

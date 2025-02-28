//! Shute (short for **sh**ader comp**ute**) is a library that simplifies
//! the use of [wgpu](https://github.com/gfx-rs/wgpu) for general-purpose
//! compute applications.
#![warn(missing_docs)]

mod buffer;
mod device;
mod instance;
mod types;

pub use buffer::{Buffer, BufferInit, BufferType};
pub use device::{Device, LimitType};
pub use encase::ShaderType;
pub use instance::Instance;
pub use types::*;

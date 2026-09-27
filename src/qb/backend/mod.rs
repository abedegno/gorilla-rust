//! The platform half of the runtime. Exactly one backend is compiled in: a
//! native window and sound card, or (from the browser build onwards) a
//! canvas and Web Audio. Everything above this module is platform-free.

#[cfg(not(target_arch = "wasm32"))]
mod clock;
#[cfg(not(target_arch = "wasm32"))]
pub mod speaker;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
mod native;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
pub use native::*;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

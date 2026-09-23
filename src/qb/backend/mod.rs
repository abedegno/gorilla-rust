//! The platform half of the runtime. Exactly one backend is compiled in: a
//! native window and sound card, or (from the browser build onwards) a
//! canvas and Web Audio. Everything above this module is platform-free.

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

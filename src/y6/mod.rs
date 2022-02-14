//! This part of the library works only for Y6 version of the asciicker
//!
//! Y6 version commit hash: 80708c9ca5f0ea8539653bb632082ce38b103903

/// # Bot module
/// Bot module is supposed to provide highest level of abstraction and allow easy creation
/// of bots.
///
/// Look in `examples/` directory for examples.
#[cfg(feature = "bot")]
pub mod bot;
/// # Packets module
/// Packets module is supposed to provide the most basic abstractions around asciicker packets
/// and conversion from and into bytes for them.
///
/// In theory this module can be used to create not only bots, but also full clients and servers.
#[cfg(feature = "packets")]
pub mod packets;
#[cfg(any(feature = "bot", feature = "packets"))]
/// # Prelude module
/// Prelude module includes basically every other module of the library in it.
pub mod prelude;
#[cfg(any(feature = "bot", feature = "packets"))]
/// # Utilities module
/// Shouldn't be used directly, only used internally for error types and similar.
pub mod utils;

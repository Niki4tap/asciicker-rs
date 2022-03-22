//! This part of the library works only for Y6 version of the asciicker
//!
//! Y6 version commit hash: 80708c9ca5f0ea8539653bb632082ce38b103903

/// # Bot module
/// Bot module is supposed to provide highest level of abstraction and allow easy creation
/// of bots.
///
/// # Examples
///
/// ## Simple chat logger:
///
/// ```rust
/// use asciicker_rs::callback;
/// use asciicker_rs::macro_rules_attribute::apply;
/// use asciicker_rs::y6::prelude::*;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// #[tokio::main]
/// async fn main() {
///     let mut bot = Bot::new("player", "ws://asciicker.com/ws/y6/", true);
///     bot.on_talk(talk_callback);
///     let (threads, _data) = match bot.run().await {
///         Err(e) => panic!("Failed to run the bot: {:?}", e),
///         Ok(stuff) => stuff,
///     };
///     println!("{:?}", threads.0.thread.await);
/// }
///
/// #[apply(callback!)]
/// pub async fn talk_callback(
///     talk_brc: TalkBroadcast,
///     _: Arc<Mutex<Player>>,
///     _: Arc<Mutex<World>>,
///     _: MessageSender,
/// ) -> BotResult {
///     println!("{:?}", talk_brc.str);
///     Ok(())
/// }
/// ```
///
/// Look in `examples/` directory more for examples.
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

use super::packets::{
    Bytes, ExitBroadcast, JoinBroadcast, JoinRequest, JoinResponse, LagStamp, PlayerPose,
    PoseBroadcast, PoseRequest, RawJoinResponse, TalkBroadcast, TalkRequest,
};
use super::utils::RuntimeError;

use std::{
    ffi::CString, future::Future, mem::swap, pin::Pin, sync::Arc, thread::sleep, time::Duration,
};

use crossbeam::channel::{unbounded, Sender as channel_Sender};
use futures_util::{SinkExt, StreamExt};
use macro_rules_attribute::apply;
use tokio::{sync::Mutex, task::JoinHandle, time::Instant};
use tokio_tungstenite::tungstenite::Message as ws_Message;

/// Result type for callbacks ([`JoinCallback`], [`ExitCallback`], [`PoseCallback`], [`TalkCallback`]), internal functions ([`patch_world`]...).
pub type BotResult = Result<(), RuntimeError>;
/// Type alias for two main connection threads.
pub type ConnectionThread = JoinHandle<Result<(), RuntimeError>>;
/// Type alias for sender handle of the message channel.
pub type MessageSender = Arc<channel_Sender<String>>;
/// Box-pinned [`BotResult`].
pub type FutureBotResult = Pin<Box<dyn Future<Output = BotResult> + Send>>;
/// Type alias for join callback.
pub type JoinCallback =
    fn(JoinBroadcast, Arc<Mutex<Player>>, Arc<Mutex<World>>, MessageSender) -> FutureBotResult;
/// Type alias for exit callback.
pub type ExitCallback =
    fn(ExitBroadcast, Arc<Mutex<Player>>, Arc<Mutex<World>>, MessageSender) -> FutureBotResult;
/// Type alias for pose callback.
pub type PoseCallback =
    fn(PoseBroadcast, Arc<Mutex<Player>>, Arc<Mutex<World>>, MessageSender) -> FutureBotResult;
/// Type alias for talk callback.
pub type TalkCallback =
    fn(TalkBroadcast, Arc<Mutex<Player>>, Arc<Mutex<World>>, MessageSender) -> FutureBotResult;
/// Type alias for main bot data
pub type BotData = (Arc<Mutex<Player>>, Arc<Mutex<World>>, MessageSender);

/// Middle level abstraction.
///
/// Represents an asciicker player
///
/// Not used internally, but created by [`Bot::run`] and passed into callbacks + main bot function
/// as representation of the bot in the asciicker world.
///
/// There is also [`Vec<Player>`] in [`World`] that represents all current players
/// (excluding the bot) and managed by [`Receiver`] thread.
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Player {
    /// Nickname
    pub nickname: String,
    /// Current position + animation + sprite
    pub pose: PlayerPose,
    /// ID
    pub id: u16,
}

/// Middle level abstraction.
///
/// Represents a message sent by someone in asciicker
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Message {
    /// Contents of the message
    pub content: String,
    /// Author id
    pub author: u16,
    /// When the message was sent
    pub when: Instant,
}

impl Message {
    /// Creates a new instance of [`Message`].
    pub fn new<S: Into<String>>(content: S, author: u16, when: Instant) -> Self {
        Self {
            content: content.into(),
            author,
            when,
        }
    }
}

/// Middle level abstraction.
///
/// Represents any asciicker world.
///
/// Not used internally, but created by [`Bot::run`] and updated by the [`Receiver`] thread.
/// Main purpose is to give the user of the library an accurate representation of what is happening.
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct World {
    /// Max amount of client the server supports.
    pub max_clients: u8,
    /// Current clients
    pub clients: Vec<Player>,
    /// Stack of messages, need to be popped manually
    pub messages: Vec<Message>,
    /// [`LagStamp`]
    pub lag: LagStamp,
}

/// A high-level abstraction function that is used
/// internally by the receiver thread
/// to patch the [`World`] by some packet from server.
///
/// For example, if this function receives a [`JoinBroadcast`],
/// it will add a new [`Player`] to the [`World`] and call
/// [`JoinCallback`] that was passed in.
///
/// [`World`]: ./struct.World.html
/// [`Player`]: ./struct.Player.html
/// [`JoinBroadcast`]: ../packets/struct.JoinBroadcast.html
/// [`JoinCallback`]: ./type.JoinCallback.html
// FIXME: This function should not call callbacks if there is none.
// Right now we just supply `default_join` and similar, but
// really we shouldn't create and call them at all
// if we don't need to.
pub async fn patch_world(
    callbacks: Arc<(JoinCallback, ExitCallback, PoseCallback, TalkCallback)>,
    data: Bytes,
    world: Arc<Mutex<World>>,
    bot: Arc<Mutex<Player>>,
    replace_invalid_utf8: bool,
    sender: MessageSender,
) -> BotResult {
    match data[0] {
        /* Accept only stuff we care about, aka broadcasts */
        b'j' => {
            // Someone has joined
            let join_brc: JoinBroadcast = match data.try_into() {
                Err(e) => return Err(RuntimeError::from_string(format!("{:?}", e))),
                Ok(brc) => brc,
            };
            match (&callbacks.0)(
                join_brc.clone(),
                Arc::clone(&bot),
                Arc::clone(&world),
                sender,
            )
            .await
            {
                Err(e) => return Err(e),
                _ => {}
            }
            let nickname = match replace_invalid_utf8 {
                true => join_brc
                    .name
                    .to_string_lossy()
                    .into_owned()
                    .replace('\u{0}', ""),
                false => join_brc.name.to_string_lossy().into_owned(),
            };
            let mut world = world.lock().await;
            world.clients.push(Player {
                nickname,
                pose: join_brc.player_pose,
                id: join_brc.id,
            });
        }

        b'e' => {
            // Someone has left
            let exit_brc: ExitBroadcast = match data.try_into() {
                Err(e) => return Err(RuntimeError::from_string(format!("{:?}", e))),
                Ok(brc) => brc,
            };
            match (&callbacks.1)(
                exit_brc.clone(),
                Arc::clone(&bot),
                Arc::clone(&world),
                sender,
            )
            .await
            {
                Err(e) => return Err(e),
                _ => {}
            }
            let mut world = world.lock().await;
            let idx = world
                .clients
                .iter()
                .position(|c| c.id == exit_brc.id)
                .unwrap();
            world.clients.remove(idx);
        }

        b'p' => {
            // Someone has moved or their pose changed for any reason
            let pose_brc: PoseBroadcast = match data.try_into() {
                Err(e) => return Err(RuntimeError::from_string(format!("{:?}", e))),
                Ok(brc) => brc,
            };
            match (&callbacks.2)(
                pose_brc.clone(),
                Arc::clone(&bot),
                Arc::clone(&world),
                sender,
            )
            .await
            {
                Err(e) => return Err(e),
                _ => {}
            }
            let mut world = world.lock().await;
            let mut client = match world.clients.iter_mut().find(|c| c.id == pose_brc.id) {
                Some(v) => v,
                None => return Ok(()),
            };
            client.pose = pose_brc.player_pose;
        }

        b't' => {
            // Someone has said something
            let talk_brc: TalkBroadcast = match data.try_into() {
                Err(e) => return Err(RuntimeError::from_string(format!("{:?}", e))),
                Ok(brc) => brc,
            };
            match (&callbacks.3)(
                talk_brc.clone(),
                Arc::clone(&bot),
                Arc::clone(&world),
                sender,
            )
            .await
            {
                Err(e) => return Err(e),
                _ => {}
            }
            let content = match replace_invalid_utf8 {
                true => talk_brc
                    .str
                    .to_string_lossy()
                    .into_owned()
                    .replace('\u{0}', ""),
                false => talk_brc.str.to_string_lossy().into_owned(),
            };
            world
                .lock()
                .await
                .messages
                .push(Message::new(content, talk_brc.id, Instant::now()));
        }

        _ => {} // Don't care
    }

    Ok(())
}

/// Macro to transform `async fn` to return
/// `Pin<Box<impl Future<Output=T>>>` instead of
/// `impl Future<Output=T>`
/// and is required for functions which are planned to be
/// passed as an argument to [`Bot::on_talk`] or similar methods.
///
/// Stolen from [here](https://users.rust-lang.org/t/how-to-store-async-function-pointer/38343/4)
/// , thanks to [Yandros](https://users.rust-lang.org/u/Yandros).
#[macro_export]
macro_rules! callback {(
    $( #[$attr:meta] )* // includes doc strings
    $pub:vis
    async
    fn $fname:ident( $($args:tt)* ) $(-> $Ret:ty)?
    {
        $($body:tt)*
    }
) => (
    $( #[$attr] )*
    #[allow(unused_parens)]
    $pub
    fn $fname( $($args)* ) -> ::std::pin::Pin<::std::boxed::Box<
        dyn ::std::future::Future<Output = ($($Ret)?)>
            + ::std::marker::Send
    >>
    {
        ::std::boxed::Box::pin(async move { $($body)* })
    }
)}

/// Describes a receiver thread.
///
/// Receiver thread is last of the two threads created on [`Bot::run`]
/// and serves a purpose of receiving websocket messages
/// from specified asciicker server and [patching the world].
///
/// [patching the world]: ./fn.patch_world.html
pub struct Receiver {
    /// Thread [`JoinHandle`]
    pub thread: ConnectionThread,
    /// `true` if thread is still alive
    pub is_finished: Arc<Mutex<bool>>,
}

/// Describes a sender thread.
///
/// Sender thread is first of the two threads created on [`Bot::run`]
/// and serves a purpose of sending pose requests
/// (required to get broadcasts) and sending talk requests when needed.
///
/// Sender has a receiver handle of message channel, so anyone with a [sender
/// handle] (callbacks, main function) can push those messages in and next time sender thread wants to
/// send a pose request, it will also send the requested messages from the channel.
///
/// [sender handle]: ./type.MessageSender.html
pub struct Sender {
    /// Thread [`JoinHandle`]
    pub thread: ConnectionThread,
    /// `true` if thread is still alive
    pub is_finished: Arc<Mutex<bool>>,
}

/// Provides highest level of abstraction.
///
/// Can be easily constructed with [`Bot::new`] and ran with [`Bot::run`].
///
/// # Examples
///
/// ## Creating a new bot and running it:
/// ```
/// use asciicker_rs::y6::prelude::*;
///
/// let bot = Bot::new("bot", "ws://asciicker.com/ws/y6/", true);
///
/// bot.run();
/// loop {}
/// ```
pub struct Bot {
    nickname: String,
    join_callback: Option<JoinCallback>,
    exit_callback: Option<ExitCallback>,
    pose_callback: Option<PoseCallback>,
    talk_callback: Option<TalkCallback>,
    replace_invalid_utf8: bool,
    address: String,
}

impl Bot {
    /// Constructs a new [`Bot`] instance.
    pub fn new<S: Into<String>>(nickname: S, address: S, replace_invalid_utf8: bool) -> Self {
        let nickname = nickname.into();
        let address = address.into();
        debug_assert!(
            nickname.len() <= 31,
            "Bot's name cannot be longer than 31 character"
        );
        Self {
            nickname,
            join_callback: None,
            exit_callback: None,
            pose_callback: None,
            talk_callback: None,
            replace_invalid_utf8,
            address,
        }
    }

    /// Replaces [`JoinCallback`] and returns [`Some(JoinCallback)`] if any was set already.
    /// [`Some(JoinCallback)`]: [Option::Some]
    pub fn on_join(&mut self, callback: JoinCallback) -> Option<JoinCallback> {
        let mut callback = Some(callback);
        swap(&mut callback, &mut self.join_callback);
        callback
    }

    /// Replaces [`ExitCallback`] and returns [`Some(ExitCallback)`] if any was set already.
    /// [`Some(ExitCallback)`]: [Option::Some]
    pub fn on_exit(&mut self, callback: ExitCallback) -> Option<ExitCallback> {
        let mut callback = Some(callback);
        swap(&mut callback, &mut self.exit_callback);
        callback
    }

    /// Replaces [`PoseCallback`] and returns [`Some(PoseCallback)`] if any was set already.
    /// [`Some(PoseCallback)`]: [Option::Some]
    pub fn on_pose(&mut self, callback: PoseCallback) -> Option<PoseCallback> {
        let mut callback = Some(callback);
        swap(&mut callback, &mut self.pose_callback);
        callback
    }

    /// Replaces [`TalkCallback`] and returns [`Some(TalkCallback)`] if any was set already.
    /// [`Some(TalkCallback)`]: [Option::Some]
    pub fn on_talk(&mut self, callback: TalkCallback) -> Option<TalkCallback> {
        let mut callback = Some(callback);
        swap(&mut callback, &mut self.talk_callback);
        callback
    }

    /// Runs the bot.
    ///
    /// Spawns two threads: [`Receiver`], [`Sender`] and returns them with [`BotData`] if connecting was successful.
    pub async fn run(self) -> Result<((Receiver, Sender), BotData), RuntimeError> {
        let (mut ws_s, mut ws_r) = match tokio_tungstenite::connect_async(self.address).await {
            Ok(ws) => ws.0.split(),
            Err(e) => {
                return Err(RuntimeError::from_string(format!(
                    "Connection failed: {:?}",
                    e
                )))
            }
        };
        let join_req: Bytes = JoinRequest {
            name: match CString::new(self.nickname.clone()) {
                Ok(s) => s,
                Err(e) => {
                    return Err(RuntimeError::from_string(format!(
                        "Failed to make new CString: {:?}",
                        e
                    )))
                }
            },
        }
        .into();
        ws_s.send(ws_Message::Binary(join_req)).await.unwrap();
        let join_rsp = JoinResponse::from(
            RawJoinResponse::try_from(match ws_r.next().await {
                Some(message) => match message.unwrap() {
                    ws_Message::Binary(data) => data,
                    _ => panic!("Server returned unknown data."),
                },
                None => panic!("Server dropped connection"),
            })
            .unwrap(),
        );
        let (tx, rx) = unbounded();
        let rx = Arc::new(rx);
        let tx = Arc::new(tx);
        let bot = Arc::new(Mutex::new(Player {
            nickname: self.nickname,
            pose: Default::default(),
            id: join_rsp.id,
        }));
        let world = Arc::new(Mutex::new(World {
            max_clients: join_rsp.max_clients,
            clients: vec![],
            messages: vec![],
            lag: [0u8; 3],
        }));
        let s_bot = Arc::clone(&bot);
        let sender_finished = Arc::new(Mutex::new(false));
        let _sender_finished = Arc::clone(&sender_finished);
        let a_rx = Arc::clone(&rx);
        let sender = tokio::spawn(async move {
            loop {
                match ws_s
                    .send(ws_Message::Binary(
                        PoseRequest {
                            player_pose: s_bot.lock().await.pose.clone(),
                        }
                        .into(),
                    ))
                    .await
                {
                    Err(e) => {
                        *sender_finished.lock().await = true;
                        return Err(RuntimeError::from_string(format!("{:?}", e)));
                    }
                    _ => {}
                };
                while let Ok(m) = Arc::clone(&a_rx).try_recv() {
                    match ws_s
                        .send(ws_Message::Binary(
                            TalkRequest {
                                str: match CString::new(m) {
                                    Ok(b) => b,
                                    Err(e) => {
                                        *sender_finished.lock().await = true;
                                        return Err(RuntimeError::from_string(format!(
                                            "CString::new failed: {:?}",
                                            e
                                        )));
                                    }
                                },
                            }
                            .into(),
                        ))
                        .await
                    {
                        Err(e) => {
                            *sender_finished.lock().await = true;
                            return Err(RuntimeError::from_string(format!("{:?}", e)));
                        }
                        Ok(_) => {}
                    };
                }
                sleep(Duration::from_millis(10));
            }
        });
        let w = Arc::clone(&world);
        let b = Arc::clone(&bot);
        let callbacks = Arc::new((
            match self.join_callback {
                Some(f) => f,
                None => default_join,
            },
            match self.exit_callback {
                Some(f) => f,
                None => default_exit,
            },
            match self.pose_callback {
                Some(f) => f,
                None => default_pose,
            },
            match self.talk_callback {
                Some(f) => f,
                None => default_talk,
            },
        ));
        let receiver_finished = Arc::new(Mutex::new(false));
        let _receiver_finished = Arc::clone(&receiver_finished);
        let a_tx = Arc::clone(&tx);
        let receiver = tokio::spawn(async move {
            while let Some(message) = ws_r.next().await {
                match message {
                    Ok(m) => match m {
                        ws_Message::Binary(data) => {
                            match patch_world(
                                Arc::clone(&callbacks),
                                data,
                                Arc::clone(&w),
                                Arc::clone(&b),
                                self.replace_invalid_utf8,
                                Arc::clone(&a_tx),
                            )
                            .await
                            {
                                Err(e) => {
                                    *receiver_finished.lock().await = true;
                                    return Err(RuntimeError::from_string(e.to_string()));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    Err(e) => {
                        *receiver_finished.lock().await = true;
                        return Err(RuntimeError::from_string(e.to_string()));
                    }
                }
            }
            Ok(())
        });
        let main_world = Arc::clone(&world);
        let main_bot = Arc::clone(&bot);
        let main_sender = Arc::clone(&tx);
        Ok((
            (
                Receiver {
                    thread: receiver,
                    is_finished: Arc::clone(&_receiver_finished),
                },
                Sender {
                    thread: sender,
                    is_finished: Arc::clone(&_sender_finished),
                },
            ),
            (main_bot, main_world, main_sender),
        ))
    }
}

#[doc(hidden)]
#[apply(callback!)]
async fn default_join(
    _: JoinBroadcast,
    _: Arc<Mutex<Player>>,
    _: Arc<Mutex<World>>,
    _: MessageSender,
) -> BotResult {
    Ok(())
}

#[doc(hidden)]
#[apply(callback!)]
async fn default_exit(
    _: ExitBroadcast,
    _: Arc<Mutex<Player>>,
    _: Arc<Mutex<World>>,
    _: MessageSender,
) -> BotResult {
    Ok(())
}

#[doc(hidden)]
#[apply(callback!)]
async fn default_pose(
    _: PoseBroadcast,
    _: Arc<Mutex<Player>>,
    _: Arc<Mutex<World>>,
    _: MessageSender,
) -> BotResult {
    Ok(())
}

#[apply(callback!)]
async fn default_talk(
    _: TalkBroadcast,
    _: Arc<Mutex<Player>>,
    _: Arc<Mutex<World>>,
    _: MessageSender,
) -> BotResult {
    Ok(())
}

#![allow(non_camel_case_types)]

use super::utils::{first_nul, PacketParseError};

use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::mem::size_of;

use concat_idents::concat_idents;

// TODO: Add tests

/// Type alias for position
pub type Position = [f32; 3];
/// Type alias for lag stamp
pub type LagStamp = [u8; 3];
/// Type alias for bytes
pub type Bytes = Vec<u8>;

/// Helper, that contains data, about player's pose: position + frame + animation + sprite...
#[derive(Debug, Default, Clone, PartialOrd, PartialEq)]
pub struct PlayerPose {
    /// Animation
    pub animation: u8,
    /// Frame
    pub frame: u8,
    /// Action or mount (wolf)
    pub action_or_mount: u8,
    /// Position
    pub position: Position,
    /// Direction
    pub direction: f32,
    /// Current sprite
    pub sprite: u16,
}

/// Size of the [`PlayerPose`] struct
pub const PLAYER_POSE_SIZE: usize = size_of::<u8>() * 3 + size_of::<f32>() * 4 + size_of::<u16>();

impl Into<Bytes> for PlayerPose {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.animation);
        b.push(self.frame);
        b.push(self.action_or_mount);
        for coord in self.position {
            b.extend_from_slice(&coord.to_ne_bytes());
        }
        b.extend_from_slice(&self.direction.to_ne_bytes());
        b.extend_from_slice(&self.sprite.to_ne_bytes());
        b
    }
}

impl TryFrom<Bytes> for PlayerPose {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != PLAYER_POSE_SIZE {
            return Err(PacketParseError::SizeMismatch(
                value.len(),
                PLAYER_POSE_SIZE,
            ));
        }

        Ok(PlayerPose {
            animation: value[0],
            frame: value[1],
            action_or_mount: value[2],
            position: [
                f32::from_ne_bytes([value[3], value[4], value[5], value[6]]),
                f32::from_ne_bytes([value[7], value[8], value[9], value[10]]),
                f32::from_ne_bytes([value[11], value[12], value[13], value[14]]),
            ],
            direction: f32::from_ne_bytes([value[15], value[16], value[17], value[18]]),
            sprite: u16::from_ne_bytes([value[19], value[20]]),
        })
    }
}

// Raw packets:

/// Provides lowest level of abstraction.
///
/// Represents raw join request sent from client to server.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L69>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`JoinRequest`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawJoinRequest {
    /// Token: 'J'
    pub token: u8,
    /// C-like string, array of characters representing the name
    pub name: [u8; 31],
}

/// Provides lowest level of abstraction.
///
/// Represents raw join response sent from server to client.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L75>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`JoinResponse`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawJoinResponse {
    /// Token: 'j'
    pub token: u8,
    /// Max clients
    pub max_clients: u8,
    /// ID of the newly joined player
    pub id: u16,
}

/// Provides lowest level of abstraction.
///
/// Represents raw join broadcast sent from server to clients.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L82>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`JoinBroadcast`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawJoinBroadcast {
    /// Token: 'j'
    ///
    /// From [original comments](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L84>): (theres collision with STRUCT_RSP_JOIN, but RSP is sent in sync, only once prior to any broadcast)
    pub token: u8,
    /// Pose of the newly joined player
    pub player_pose: PlayerPose,
    /// ID of the newly joined player
    pub id: u16,
    /// C-like string, array of characters representing the name of the newly joined player
    pub name: [u8; 32],
}

/// Provides lowest level of abstraction.
///
/// Represents raw exit broadcast sent from server to clients.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L95>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`ExitBroadcast`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawExitBroadcast {
    /// Token: 'e'
    pub token: u8,
    /// Not actually used. Necessary for padding.
    pub _padding: u8,
    /// ID of the player who just exited
    pub id: u16,
}

/// Provides lowest level of abstraction.
///
/// Represents raw pose request sent from client to server.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L102>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`PoseRequest`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawPoseRequest {
    /// Token: 'P'
    pub token: u8,
    /// Current pose of the player
    pub player_pose: PlayerPose,
}

/// Provides lowest level of abstraction.
///
/// Represents raw exit broadcast sent from server to clients.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L113>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`PoseBroadcast`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawPoseBroadcast {
    /// Token: 'p'
    pub token: u8,
    /// Current pose of the player who sent the pose request
    pub player_pose: PlayerPose,
    /// ID of the player who sent the pose request
    pub id: u16,
}

/// Provides lowest level of abstraction.
///
/// Represents raw talk request sent from client to server.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L125>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`TalkRequest`]
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct RawTalkRequest {
    /// Token: 'T'
    pub token: u8,
    /// Length of the str field
    pub len: u8,
    /// Contents of the message
    ///
    /// This field is different from the original definition, but the original definition
    /// marks this as byte array with length of 256 with comment of: "trim to actual size!"
    ///
    /// So this library trims the string for you and converts it into a [`CString`]
    pub str: CString,
}

/// Provides lowest level of abstraction.
///
/// Represents raw talk broadcast sent from server to clients.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L132>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`TalkBroadcast`]
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct RawTalkBroadcast {
    /// Token: 't'
    pub token: u8,
    /// Length of the str field
    pub len: u8,
    /// ID of the player who sent the request
    pub id: u16,
    /// Contents of the message
    ///
    /// This field is different from the original definition, but the original definition
    /// marks this as byte array with length of 256 with comment of: "trim to actual size!"
    ///
    /// So this library trims the string for you and converts it into a [`CString`]
    pub str: CString,
}

/// Provides lowest level of abstraction.
///
/// Represents raw lag request sent from client to server.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L140>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`LagRequest`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawLagRequest {
    /// Token: 'L'
    pub token: u8,
    /// Lag stamp
    pub stamp: LagStamp,
}

/// Provides lowest level of abstraction.
///
/// Represents raw lag response sent from server to client.
///
/// Definition basically copied from here: <https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h#L146>
///
/// Can be transformed [`from`](std::convert::From) [`Bytes`] and [`into`](std::convert::Into) [`LagResponse`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct RawLagResponse {
    /// Token: 'l'
    pub token: u8,
    /// Lag stamp
    pub stamp: LagStamp,
}

// Raw structs aliases, like in C code:

/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_REQ_JOIN = RawJoinRequest;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_RSP_JOIN = RawJoinResponse;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_BRC_JOIN = RawJoinBroadcast;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_BRC_EXIT = RawExitBroadcast;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_REQ_POSE = RawPoseRequest;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_BRC_POSE = RawPoseBroadcast;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_REQ_TALK = RawTalkRequest;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_BRC_TALK = RawTalkBroadcast;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_REQ_LAG = RawLagRequest;
/// Type alias for packets like in the [original code](<https://github.com/msokalski/asciicker/blob/80708c9ca5f0ea8539653bb632082ce38b103903/network.h>)
pub type STRUCT_RSP_LAG = RawLagResponse;

// Clean packets:

/// Low level abstraction.
///
/// Represents clean version of the join request, sent from client to server.
///
/// Can be transformed [`from`](std::convert::From) [`RawJoinRequest`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct JoinRequest {
    /// Name of the player who requests to join the server
    pub name: CString,
}

/// Low level abstraction.
///
/// Represents clean version of the join response, sent from server to client.
///
/// Can be transformed [`from`](std::convert::From) [`RawJoinResponse`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct JoinResponse {
    /// Max clients
    pub max_clients: u8,
    /// ID
    pub id: u16,
}

/// Low level abstraction.
///
/// Represents clean version of the join broadcast, sent from server to clients.
///
/// Can be transformed [`from`](std::convert::From) [`RawJoinBroadcast`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct JoinBroadcast {
    /// Pose of the newly joined player
    pub player_pose: PlayerPose,
    /// ID of the newly joined player
    pub id: u16,
    /// Name of the newly joined player
    pub name: CString,
}

/// Low level abstraction.
///
/// Represents clean version of the exit broadcast, sent from server to clients.
///
/// Can be transformed [`from`](std::convert::From) [`RawExitBroadcast`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct ExitBroadcast {
    /// ID of the player who just exited
    pub id: u16,
}

/// Low level abstraction.
///
/// Represents clean version of the pose request, sent from client to server.
///
/// Can be transformed [`from`](std::convert::From) [`RawPoseRequest`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct PoseRequest {
    /// Current pose of the player
    pub player_pose: PlayerPose,
}

/// Low level abstraction.
///
/// Represents clean version of the pose broadcast, sent from server to clients.
///
/// Can be transformed [`from`](std::convert::From) [`RawPoseBroadcast`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct PoseBroadcast {
    /// Current pose of the player who sent the pose request
    pub player_pose: PlayerPose,
    /// ID of the player who sent the pose request
    pub id: u16,
}

/// Low level abstraction.
///
/// Represents clean version of the talk request, sent from client to server.
///
/// Can be transformed [`from`](std::convert::From) [`RawTalkRequest`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct TalkRequest {
    /// Message contents
    pub str: CString,
}

/// Low level abstraction.
///
/// Represents clean version of the talk broadcast, sent from server to clients.
///
/// Can be transformed [`from`](std::convert::From) [`RawTalkBroadcast`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct TalkBroadcast {
    /// ID of the player who sent the broadcast
    pub id: u16,
    /// Message contents
    pub str: CString,
}

/// Low level abstraction.
///
/// Represents clean version of the lag request, sent from client to server.
///
/// Can be transformed [`from`](std::convert::From) [`RawLagRequest`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct LagRequest {
    /// Lag stamp
    pub stamp: LagStamp,
}

/// Low level abstraction.
///
/// Represents clean version of the lag response, sent from server to client.
///
/// Can be transformed [`from`](std::convert::From) [`RawLagResponse`] and [`into`](std::convert::Into) [`Bytes`]
#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct LagResponse {
    /// Lag stamp
    pub stamp: LagStamp,
}

// Sizes:
/// Size of the [`RawJoinRequest`] struct in C
pub const JOIN_REQ_SIZE: usize = size_of::<u8>() + size_of::<u8>() * 31;
/// Size of the [`RawJoinResponse`] struct in C
pub const JOIN_RSP_SIZE: usize = size_of::<u8>() * 2 + size_of::<u16>();
/// Size of the [`RawJoinBroadcast`] struct in C
pub const JOIN_BRC_SIZE: usize =
    size_of::<u8>() * 4 + size_of::<f32>() * 4 + size_of::<u16>() * 2 + size_of::<u8>() * 32;
/// Size of the [`RawExitBroadcast`] struct in C
pub const EXIT_BRC_SIZE: usize = size_of::<u8>() * 2 + size_of::<u16>();
/// Size of the [`RawPoseRequest`] struct in C
pub const POSE_REQ_SIZE: usize = size_of::<u8>() * 4 + size_of::<f32>() * 4 + size_of::<u16>();
/// Size of the [`RawPoseBroadcast`] struct in C
pub const POSE_BRC_SIZE: usize = size_of::<u8>() * 4 + size_of::<f32>() * 4 + size_of::<u16>() * 2;
/// Max possible size of the [`RawTalkRequest`] struct in C
pub const TOTAL_TALK_REQ_SIZE: usize = size_of::<u8>() * 258;
/// Max possible size of the [`RawTalkBroadcast`] struct in C
pub const TOTAL_TALK_BRC_SIZE: usize = size_of::<u8>() * 258 + size_of::<u16>();
/// Size of the [`RawLagRequest`] struct in C
pub const LAG_REQ_SIZE: usize = size_of::<u8>() * 4;
/// Size of the [`RawLagResponse`] struct in C
pub const LAG_RSP_SIZE: usize = size_of::<u8>() * 4;

// Bytes to raw packet structs:

impl TryFrom<Bytes> for RawJoinRequest {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != JOIN_REQ_SIZE {
            return Err(PacketParseError::SizeMismatch(JOIN_REQ_SIZE, value.len()));
        }
        Ok(RawJoinRequest {
            token: value[0],
            name: value[1..(1 + 31)].try_into().unwrap(),
        })
    }
}

impl TryFrom<Bytes> for RawJoinResponse {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != JOIN_RSP_SIZE {
            return Err(PacketParseError::SizeMismatch(JOIN_RSP_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            max_clients: value[1],
            id: u16::from_ne_bytes([value[2], value[3]]),
        })
    }
}

impl TryFrom<Bytes> for RawJoinBroadcast {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != JOIN_BRC_SIZE {
            return Err(PacketParseError::SizeMismatch(JOIN_BRC_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            player_pose: PlayerPose {
                animation: value[1],
                frame: value[2],
                action_or_mount: value[3],
                position: [
                    f32::from_ne_bytes([value[4], value[5], value[6], value[7]]),
                    f32::from_ne_bytes([value[8], value[9], value[10], value[11]]),
                    f32::from_ne_bytes([value[12], value[13], value[14], value[15]]),
                ],
                direction: f32::from_ne_bytes([value[16], value[17], value[18], value[19]]),
                sprite: u16::from_ne_bytes([value[22], value[23]]),
            },
            id: u16::from_ne_bytes([value[20], value[21]]),
            name: value[24..24 + 32].try_into().unwrap(),
        })
    }
}

impl TryFrom<Bytes> for RawExitBroadcast {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != EXIT_BRC_SIZE {
            return Err(PacketParseError::SizeMismatch(EXIT_BRC_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            _padding: value[1],
            id: u16::from_le_bytes([value[2], value[3]]),
        })
    }
}

impl TryFrom<Bytes> for RawPoseRequest {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != POSE_REQ_SIZE {
            return Err(PacketParseError::SizeMismatch(POSE_REQ_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            player_pose: PlayerPose::try_from(value[1..=21].to_vec())?,
        })
    }
}

impl TryFrom<Bytes> for RawPoseBroadcast {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != POSE_BRC_SIZE {
            return Err(PacketParseError::SizeMismatch(POSE_BRC_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            player_pose: PlayerPose::try_from(value[1..=21].to_vec())?,
            id: u16::from_ne_bytes([value[22], value[23]]),
        })
    }
}

impl TryFrom<Bytes> for RawTalkRequest {
    type Error = PacketParseError;

    fn try_from(mut value: Bytes) -> Result<Self, Self::Error> {
        let len = match first_nul(&value[2..]) {
            None => return Err(PacketParseError::NoNullByte(value[2..].to_vec())),
            Some(l) => l,
        };
        Ok(Self {
            token: value[0],
            len: value[1],
            str: unsafe { CString::from_vec_unchecked(Into::<Vec<u8>>::into(&mut value[2..len])) },
        })
    }
}

impl TryFrom<Bytes> for RawTalkBroadcast {
    type Error = PacketParseError;

    fn try_from(mut value: Bytes) -> Result<Self, Self::Error> {
        Ok(Self {
            token: value[0],
            len: value[1],
            id: u16::from_ne_bytes([value[2], value[3]]),
            str: unsafe { CString::from_vec_unchecked(Into::<Vec<u8>>::into(&mut value[4..])) },
        })
    }
}

impl TryFrom<Bytes> for RawLagRequest {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != LAG_REQ_SIZE {
            return Err(PacketParseError::SizeMismatch(LAG_REQ_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            stamp: [value[1], value[2], value[3]],
        })
    }
}

impl TryFrom<Bytes> for RawLagResponse {
    type Error = PacketParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != LAG_RSP_SIZE {
            return Err(PacketParseError::SizeMismatch(LAG_RSP_SIZE, value.len()));
        }
        Ok(Self {
            token: value[0],
            stamp: [value[1], value[2], value[3]],
        })
    }
}

// Raw to clean packet structs:

impl From<RawJoinRequest> for JoinRequest {
    fn from(value: RawJoinRequest) -> Self {
        let cstr = value.name.to_vec();
        Self {
            name: unsafe { CString::from_vec_unchecked(cstr[0..first_nul(&cstr).unwrap_or(32)].to_vec()) },
        }
    }
}

impl From<RawJoinResponse> for JoinResponse {
    fn from(value: RawJoinResponse) -> Self {
        Self {
            max_clients: value.max_clients,
            id: value.id,
        }
    }
}

impl From<RawJoinBroadcast> for JoinBroadcast {
    fn from(value: RawJoinBroadcast) -> Self {
        Self {
            player_pose: value.player_pose,
            id: value.id,
            name: unsafe { CString::from_vec_unchecked(value.name[0..first_nul(&value.name).unwrap_or(32)].to_vec()) },
        }
    }
}

impl From<RawExitBroadcast> for ExitBroadcast {
    fn from(value: RawExitBroadcast) -> Self {
        Self { id: value.id }
    }
}

impl From<RawPoseRequest> for PoseRequest {
    fn from(value: RawPoseRequest) -> Self {
        Self {
            player_pose: value.player_pose,
        }
    }
}

impl From<RawPoseBroadcast> for PoseBroadcast {
    fn from(value: RawPoseBroadcast) -> Self {
        Self {
            player_pose: value.player_pose,
            id: value.id,
        }
    }
}

impl From<RawTalkRequest> for TalkRequest {
    fn from(value: RawTalkRequest) -> Self {
        Self { str: value.str }
    }
}

impl From<RawTalkBroadcast> for TalkBroadcast {
    fn from(value: RawTalkBroadcast) -> Self {
        Self {
            id: value.id,
            str: value.str,
        }
    }
}

impl From<RawLagRequest> for LagRequest {
    fn from(value: RawLagRequest) -> Self {
        Self { stamp: value.stamp }
    }
}

impl From<RawLagResponse> for LagResponse {
    fn from(value: RawLagResponse) -> Self {
        Self { stamp: value.stamp }
    }
}

// Clean to raw packet structs:

impl Into<RawJoinRequest> for JoinRequest {
    fn into(self) -> RawJoinRequest {
        let mut name = [b'\0'; 31];
        let mut i = 0;
        for elem in self.name.into_bytes() {
            name[i] = elem;
            i += 1;
        }
        RawJoinRequest { token: b'J', name }
    }
}

impl Into<RawJoinResponse> for JoinResponse {
    fn into(self) -> RawJoinResponse {
        RawJoinResponse {
            token: b'j',
            max_clients: self.max_clients,
            id: self.id,
        }
    }
}

impl Into<RawJoinBroadcast> for JoinBroadcast {
    fn into(self) -> RawJoinBroadcast {
        let mut name = [b'\0'; 32];
        let mut i = 0;
        for elem in self.name.into_bytes() {
            name[i] = elem;
            i += 1;
        }
        RawJoinBroadcast {
            token: b'j',
            player_pose: self.player_pose,
            id: self.id,
            name,
        }
    }
}

impl Into<RawExitBroadcast> for ExitBroadcast {
    fn into(self) -> RawExitBroadcast {
        RawExitBroadcast {
            token: b'e',
            _padding: 0,
            id: self.id,
        }
    }
}

impl Into<RawPoseRequest> for PoseRequest {
    fn into(self) -> RawPoseRequest {
        RawPoseRequest {
            token: b'P',
            player_pose: self.player_pose,
        }
    }
}

impl Into<RawPoseBroadcast> for PoseBroadcast {
    fn into(self) -> RawPoseBroadcast {
        RawPoseBroadcast {
            token: b'p',
            player_pose: self.player_pose,
            id: self.id,
        }
    }
}

impl Into<RawTalkRequest> for TalkRequest {
    fn into(self) -> RawTalkRequest {
        let bytes = self.str.as_bytes();
        RawTalkRequest {
            token: b'T',
            len: bytes.len() as u8,
            str: self.str,
        }
    }
}

impl Into<RawTalkBroadcast> for TalkBroadcast {
    fn into(self) -> RawTalkBroadcast {
        let bytes = self.str.as_bytes();
        RawTalkBroadcast {
            token: b't',
            len: bytes.len() as u8,
            id: self.id,
            str: self.str,
        }
    }
}

impl Into<RawLagRequest> for LagRequest {
    fn into(self) -> RawLagRequest {
        RawLagRequest {
            token: b'L',
            stamp: self.stamp,
        }
    }
}

impl Into<RawLagResponse> for LagResponse {
    fn into(self) -> RawLagResponse {
        RawLagResponse {
            token: b'l',
            stamp: self.stamp,
        }
    }
}

impl Into<Bytes> for RawJoinRequest {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.extend_from_slice(&self.name);
        b
    }
}

impl Into<Bytes> for RawJoinResponse {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.push(self.max_clients);
        b.extend_from_slice(&self.id.to_ne_bytes());
        b
    }
}

impl Into<Bytes> for RawJoinBroadcast {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.push(self.player_pose.animation);
        b.push(self.player_pose.frame);
        b.push(self.player_pose.action_or_mount);
        for coord in self.player_pose.position {
            b.extend_from_slice(&coord.to_ne_bytes());
        }
        b.extend_from_slice(&self.player_pose.direction.to_ne_bytes());
        b.extend_from_slice(&self.id.to_ne_bytes());
        b.extend_from_slice(&self.player_pose.sprite.to_ne_bytes());
        b.extend_from_slice(&self.name);
        b
    }
}

impl Into<Bytes> for RawExitBroadcast {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.push(0);
        b.extend_from_slice(&self.id.to_ne_bytes());
        b
    }
}

impl Into<Bytes> for RawPoseRequest {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.push(self.player_pose.animation);
        b.push(self.player_pose.frame);
        b.push(self.player_pose.action_or_mount);
        for coord in self.player_pose.position {
            b.extend_from_slice(&coord.to_ne_bytes());
        }
        b.extend_from_slice(&self.player_pose.direction.to_ne_bytes());
        b.extend_from_slice(&self.player_pose.sprite.to_ne_bytes());
        b
    }
}

impl Into<Bytes> for RawPoseBroadcast {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.push(self.player_pose.animation);
        b.push(self.player_pose.frame);
        b.push(self.player_pose.action_or_mount);
        for coord in self.player_pose.position {
            b.extend_from_slice(&coord.to_ne_bytes());
        }
        b.extend_from_slice(&self.player_pose.direction.to_ne_bytes());
        b.extend_from_slice(&self.player_pose.sprite.to_ne_bytes());
        b.extend_from_slice(&self.id.to_ne_bytes());
        b
    }
}

impl Into<Bytes> for RawTalkRequest {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        let mut string = self.str.into_bytes_with_nul();
        string.push(b'\0'); // Additional null-byte for padding, not terminating
        b.push(self.token);
        b.push(self.len);
        b.extend(&string);
        b
    }
}

impl Into<Bytes> for RawTalkBroadcast {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        let mut string = self.str.into_bytes_with_nul();
        string.push(b'\0'); // Additional null-byte for padding, not terminating
        b.push(self.token);
        b.push(self.len);
        b.extend_from_slice(&self.id.to_ne_bytes());
        b.extend(&string);
        b
    }
}

impl Into<Bytes> for RawLagRequest {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.extend_from_slice(&self.stamp);
        b
    }
}

impl Into<Bytes> for RawLagResponse {
    fn into(self) -> Bytes {
        let mut b = Bytes::new();
        b.push(self.token);
        b.extend_from_slice(&self.stamp);
        b
    }
}

#[doc(hidden)]
macro_rules! impl_from_bytes_for_clean {
    ($($name:ident)+) => {
        $(
            impl TryFrom<Bytes> for $name {
                type Error = PacketParseError;

                fn try_from(value: Bytes) -> Result<Self, Self::Error> {
                    match <concat_idents!(id = Raw, $name { id })>::try_from(value) {
                        Err(e) => return Err(e),
                        Ok(d) => return Ok($name::from(d)),
                    }
                }
            }
        )+
    };
}

impl_from_bytes_for_clean!(JoinRequest JoinResponse JoinBroadcast ExitBroadcast PoseRequest PoseBroadcast TalkRequest TalkBroadcast LagRequest LagResponse);

#[doc(hidden)]
macro_rules! impl_into_bytes_for_clean {
    ($($name:ident)+) => {
        $(
            impl Into<Bytes> for $name {
                fn into(self) -> Bytes {
                    Into::<concat_idents!(id = Raw, $name { id })>::into(self).into()
                }
            }
        )+
    };
}

impl_into_bytes_for_clean!(JoinRequest JoinResponse JoinBroadcast ExitBroadcast PoseRequest PoseBroadcast TalkRequest TalkBroadcast LagRequest LagResponse);

// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use const_panic::{FmtArg, PanicFmt, PanicVal, unwrap_ok};
use core::{
    fmt,
    num::{NonZero, NonZeroI64},
    ops::RangeInclusive,
};
use snafu::Snafu;

use grammers_tl_types as tl;

macro_rules! non_zero {
    ($n:expr) => {
        const { ::core::num::NonZero::new($n).expect("non-zero constant") }
    };
}

/// A compact peer identifier.
/// ```
/// use std::mem::size_of;
/// assert_eq!(size_of::<grammers_session::types::PeerId>(), size_of::<i64>());
/// ```
/// The [`PeerInfo`] cached by the session for this `PeerId` may be retrieved via [`crate::Session::peer`].
///
/// The internal representation uses the Bot API Dialog ID format to
/// bit-pack both the peer's true identifier and type in a single integer.
///
/// Internally, arbitrary values outside the valid range of Bot API Dialog ID
/// may be used to represent special peer identifiers.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PeerId(NonZeroI64);

#[derive(Clone, Copy, Debug, Snafu)]
pub enum PeerIdFromSelfUserError {}

impl PanicFmt for PeerIdFromSelfUserError {
    type This = Self;
    type Kind = const_panic::IsCustomType;
    const PV_COUNT: usize = 0;
}

impl PeerIdFromSelfUserError {
    #[allow(dead_code)]
    pub const fn to_panicvals(
        self,
        _: FmtArg,
    ) -> [PanicVal<'static>; <PeerIdFromSelfUserError as PanicFmt>::PV_COUNT] {
        match self {}
    }
}

#[derive(Clone, Copy, Debug, Snafu, PanicFmt)]
pub enum PeerIdFromUserError {
    #[snafu(display("user ID is out of range"))]
    #[snafu(context(name(UserIdOutOfRangeSnafu)))]
    IdOutOfRange,
}

#[derive(Clone, Copy, Debug, Snafu, PanicFmt)]
pub enum PeerIdFromChatError {
    #[snafu(display("chat ID is out of range"))]
    #[snafu(context(name(ChatIdOutOfRangeSnafu)))]
    IdOutOfRange,
}

#[derive(Clone, Copy, Debug, Snafu, PanicFmt)]
pub enum PeerIdFromChannelError {
    #[snafu(display("channel ID is out of range"))]
    #[snafu(context(name(ChannelIdOutOfRangeSnafu)))]
    IdOutOfRange,
}

#[derive(Clone, Copy, Debug, Snafu, PanicFmt)]
pub enum BarePeerIdError {
    #[snafu(display("self-user ID not known"))]
    #[snafu(context(name(SelfUserBareIdNotKnownSnafu)))]
    SelfUserIdNotKnown,
}

#[derive(Clone, Copy, Debug, Snafu, PanicFmt)]
pub enum PeerIdError {
    #[snafu(transparent)]
    SelfUser { source: PeerIdFromSelfUserError },
    #[snafu(transparent)]
    User { source: PeerIdFromUserError },
    #[snafu(transparent)]
    Chat { source: PeerIdFromChatError },
    #[snafu(transparent)]
    Channel { source: PeerIdFromChannelError },
}

/// Witness to the session's authority from Telegram to interact with a peer.
///
/// If Telegram deems the session to already have such authority, the session may
/// be allowed to present [`PeerAuth::default`] instead of this witness. This can
/// happen when the logged-in user is a bot account, or, for user accounts, when
/// the peer being interacted with is one of its contacts.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PeerAuth(i64);

/// Ocap-style reference to a peer object, a peer object capability, bundling the identity
/// of a peer (its [`PeerId`]) with authority over it (as [`PeerAuth`]), to allow fluent use.
///
/// This type implements conversion to [`tl::enums::InputPeer`] and derivatives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PeerRef {
    /// The peer identity.
    pub id: PeerId,
    /// The authority bound to both the sibling identity and the session of the logged-in user.
    pub auth: PeerAuth,
}

/// [`PeerId`]'s kind.
///
/// The `PeerId` bitpacks this information for size reasons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PeerKind {
    /// The peer identity belongs to a [`tl::enums::User`]. May also represent [`PeerKind::UserSelf`].
    User,
    /// The peer identity belongs to a user with its [`tl::types::User::is_self`] flag set to `true`.
    UserSelf,
    /// The peer identity belongs to a [`tl::types::Chat`] or one of its derivatives.
    Chat,
    /// The peer identity belongs to a [`tl::types::Channel`] or one of its derivatives.
    Channel,
}

/// An exploded peer reference along with any known useful information about the peer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeerInfo {
    User {
        /// Bare user identifier.
        ///
        /// Despite being [`NonZeroI64`], Telegram only uses strictly positive values.
        id: NonZeroI64,
        /// Non-ambient authority bound to both the user itself and the session.
        auth: Option<PeerAuth>,
        /// Whether this user represents a bot or not.
        bot: Option<bool>,
        /// Whether this user represents the logged-in user authorized by this session or not.
        is_self: Option<bool>,
    },
    Chat {
        /// Bare chat identifier.
        ///
        /// Note that the HTTP Bot API negates this identifier to signal that it is a chat,
        /// but the true value used by Telegram's API is always strictly-positive.
        id: NonZeroI64,
    },
    Channel {
        /// Bare channel identifier.
        ///
        /// Note that the HTTP Bot API prefixes this identifier with `-100` to signal that it is a channel,
        /// but the true value used by Telegram's API is always strictly-positive.
        id: NonZeroI64,
        /// Non-ambient authority bound to both the user itself and the session.
        auth: Option<PeerAuth>,
        /// Channel kind, useful to determine what the possible permissions on it are.
        kind: Option<ChannelKind>,
    },
}

/// Additional information about a [`PeerInfo::Channel`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelKind {
    /// Value used for a channel with its [`tl::types::Channel::megagroup`] flag set to `true`.
    Megagroup,
    /// Value used for a channel with its [`tl::types::Channel::broadcast`] flag set to `true`.
    Broadcast,
    /// Value used for a channel with its [`tl::types::Channel::gigagroup`] flag set to `true`.
    Gigagroup,
}

/// Sentinel value used to represent the self-user when its true `PeerId` is unknown.
///
/// Per https://core.telegram.org/api/bots/ids:
/// > a bot API dialog ID ranges from -4000000000000 to 1099511627775
///
/// This value is not intended to be visible or persisted, so it can be changed as needed in the future.
const SELF_USER_ID: i64 = 1 << 40;
const NON_ZERO_SELF_USER_ID: NonZeroI64 = non_zero!(SELF_USER_ID);
const SELF_USER_PEER_ID: PeerId = PeerId(NON_ZERO_SELF_USER_ID);

/// https://core.telegram.org/api/bots/ids#user-ids
const MIN_USER_ID: i64 = 1;
/// https://core.telegram.org/api/bots/ids#user-ids
const MAX_USER_ID: i64 = 0xffffffffff;
/// https://core.telegram.org/api/bots/ids#user-ids
const NON_ZERO_MIN_USER_ID: NonZeroI64 = non_zero!(MIN_USER_ID);
/// https://core.telegram.org/api/bots/ids#user-ids
const NON_ZERO_MAX_USER_ID: NonZeroI64 = non_zero!(MAX_USER_ID);
/// https://core.telegram.org/api/bots/ids#user-ids
#[allow(dead_code)]
const USER_ID_RANGE: RangeInclusive<NonZeroI64> = NON_ZERO_MIN_USER_ID..=NON_ZERO_MAX_USER_ID;

/// https://core.telegram.org/api/bots/ids#chat-ids
const MIN_CHAT_ID: i64 = -999999999999;
/// https://core.telegram.org/api/bots/ids#chat-ids
const MAX_CHAT_ID: i64 = -1;
/// https://core.telegram.org/api/bots/ids#chat-ids
const NON_ZERO_MIN_CHAT_ID: NonZeroI64 = non_zero!(MIN_CHAT_ID);
/// https://core.telegram.org/api/bots/ids#chat-ids
const NON_ZERO_MAX_CHAT_ID: NonZeroI64 = non_zero!(MAX_CHAT_ID);
/// https://core.telegram.org/api/bots/ids#chat-ids
#[allow(dead_code)]
const CHAT_ID_RANGE: RangeInclusive<NonZeroI64> = NON_ZERO_MIN_CHAT_ID..=NON_ZERO_MAX_CHAT_ID;

/// https://core.telegram.org/api/bots/ids#supergroup-channel-ids
const MIN_CHANNEL_ID: i64 = -1997852516352;
/// https://core.telegram.org/api/bots/ids#supergroup-channel-ids
const MAX_CHANNEL_ID: i64 = -1000000000001;
/// https://core.telegram.org/api/bots/ids#supergroup-channel-ids
const NON_ZERO_MIN_CHANNEL_ID: NonZeroI64 = non_zero!(MIN_CHANNEL_ID);
/// https://core.telegram.org/api/bots/ids#supergroup-channel-ids
const NON_ZERO_MAX_CHANNEL_ID: NonZeroI64 = non_zero!(MAX_CHANNEL_ID);
/// https://core.telegram.org/api/bots/ids#supergroup-channel-ids
#[allow(dead_code)]
const CHANNEL_ID_RANGE: RangeInclusive<NonZeroI64> =
    NON_ZERO_MIN_CHANNEL_ID..=NON_ZERO_MAX_CHANNEL_ID;

/// https://core.telegram.org/api/bots/ids#monoforum-ids
const MIN_MONOFORUM_ID: i64 = -4000000000000;
/// https://core.telegram.org/api/bots/ids#monoforum-ids
const MAX_MONOFORUM_ID: i64 = -2002147483649;
/// https://core.telegram.org/api/bots/ids#monoforum-ids
const NON_ZERO_MIN_MONOFORUM_ID: NonZeroI64 = non_zero!(MIN_MONOFORUM_ID);
/// https://core.telegram.org/api/bots/ids#monoforum-ids
const NON_ZERO_MAX_MONOFORUM_ID: NonZeroI64 = non_zero!(MAX_MONOFORUM_ID);
/// https://core.telegram.org/api/bots/ids#monoforum-ids
#[allow(dead_code)]
const MONOFORUM_ID_RANGE: RangeInclusive<NonZeroI64> =
    NON_ZERO_MIN_MONOFORUM_ID..=NON_ZERO_MAX_MONOFORUM_ID;

/// Sentinel value used to represent empty chats.
///
/// Per https://core.telegram.org/api/bots/ids:
/// > \[…] transformed range for bot API chat dialog IDs is -999999999999 to -1 inclusively
/// >
/// > \[…] transformed range for bot API channel dialog IDs is -1997852516352 to -1000000000001 inclusively
///
/// `chat_id` parameters are in Telegram's API use the bare identifier, so there's no
/// empty constructor, but it can be mimicked by picking the value in the correct range hole.
/// This value is closer to "channel with ID 0" than "chat with ID 0", but there's no distinct
/// `-0` integer, and channels have a proper constructor for empty already
const EMPTY_CHAT_ID: i64 = -1000000000000;
const NON_ZERO_EMPTY_CHAT_ID: NonZeroI64 = non_zero!(EMPTY_CHAT_ID);

/// The ambient authority to authorize peers only when Telegram considers it valid.
///
/// See [`PeerAuth::default()`].
const AMBIENT_PEER_AUTH: PeerAuth = PeerAuth(0);

impl PeerId {
    /// Creates a peer identity for the currently-logged-in user or bot account.
    /// May panic.
    ///
    /// See [`fn@PeerId::self_user_checked`].
    pub const fn self_user() -> Self {
        unwrap_ok!(Self::self_user_checked())
    }

    /// Creates a peer identity for the currently-logged-in user or bot account.
    ///
    /// Internally, this will use a special sentinel value outside of any valid Bot API Dialog ID range.
    pub const fn self_user_checked() -> Result<Self, PeerIdFromSelfUserError> {
        Ok(SELF_USER_PEER_ID)
    }

    /// Creates a peer identity for a user or bot account. May panic.
    ///
    /// See [`fn@PeerId::user_checked`].
    pub const fn user(id: i64) -> Self {
        unwrap_ok!(match <NonZero<_>>::new(id) {
            Some(id) => Self::user_checked(id),
            None => Err(PeerIdFromUserError::IdOutOfRange),
        })
    }

    /// Creates a peer identity for a user or bot account.
    pub const fn user_checked(id: NonZeroI64) -> Result<Self, PeerIdFromUserError> {
        if let MIN_USER_ID..=MAX_USER_ID = id.get() {
            Ok(Self(id))
        } else {
            Err(PeerIdFromUserError::IdOutOfRange)
        }
    }

    /// Creates a peer identity for a small group chat.
    /// May panic.
    ///
    /// See [`fn@PeerId::chat_checked`].
    pub const fn chat(id: i64) -> Self {
        unwrap_ok!(match <NonZero<_>>::new(id) {
            Some(id) => Self::chat_checked(id),
            None => Err(PeerIdFromChatError::IdOutOfRange),
        })
    }

    /// Creates a peer identity for a small group chat.
    pub const fn chat_checked(id: NonZeroI64) -> Result<Self, PeerIdFromChatError> {
        if let Some(id) = id.checked_neg()
            && let MIN_CHAT_ID..=MAX_CHAT_ID = id.get()
        {
            Ok(Self(id))
        } else {
            Err(PeerIdFromChatError::IdOutOfRange)
        }
    }

    /// Creates a peer identity for a broadcast channel, megagroup, gigagroup or monoforum.
    /// May panic.
    ///
    /// See [`fn@PeerId::channel_checked`].
    pub const fn channel(id: i64) -> Self {
        unwrap_ok!(match <NonZero<_>>::new(id) {
            Some(id) => Self::channel_checked(id),
            None => Err(PeerIdFromChannelError::IdOutOfRange),
        })
    }

    /// Creates a peer identity for a broadcast channel, megagroup, gigagroup or monoforum.
    pub const fn channel_checked(id: NonZeroI64) -> Result<Self, PeerIdFromChannelError> {
        if let Some(id) = id.get().checked_add(1000000000000i64)
            && let Some(id) = id.checked_neg()
            && let MIN_CHANNEL_ID..=MAX_CHANNEL_ID | MIN_MONOFORUM_ID..=MAX_MONOFORUM_ID = id
            && let Some(id) = <NonZero<_>>::new(id)
        {
            Ok(Self(id))
        } else {
            Err(PeerIdFromChannelError::IdOutOfRange)
        }
    }

    /// Peer kind.
    pub const fn kind(self) -> PeerKind {
        match self.bot_api_dialog_id().get() {
            SELF_USER_ID => PeerKind::UserSelf,
            MIN_USER_ID..=MAX_USER_ID => PeerKind::User,
            MIN_CHAT_ID..=MAX_CHAT_ID => PeerKind::Chat,
            MIN_CHANNEL_ID..=MAX_CHANNEL_ID | MIN_MONOFORUM_ID..=MAX_MONOFORUM_ID => {
                PeerKind::Channel
            }
            _ => panic!("PeerId contains ID that fails smart constructors"),
        }
    }

    /// Returns the identity using the Bot API Dialog ID format.
    ///
    /// Will return an arbitrary value if [`Self::kind`] is [`PeerKind::UserSelf`].
    /// This value should not be relied on and may change between releases.
    pub const fn bot_api_dialog_id(self) -> NonZeroI64 {
        self.0
    }

    /// Unpacked peer identifier. Panics if [`Self::kind`] is [`PeerKind::UserSelf`].
    pub const fn bare_id(&self) -> Result<NonZeroI64, BarePeerIdError> {
        match (self.kind(), self.bot_api_dialog_id()) {
            (PeerKind::UserSelf, _) => Err(BarePeerIdError::SelfUserIdNotKnown),
            (PeerKind::User, id) => Ok(id),
            (PeerKind::Chat, id) => Ok(id
                .checked_neg()
                .expect("PeerId contains chat ID that fails smart constructors")),
            (PeerKind::Channel, id) => {
                if let Some(id) = id.checked_neg()
                    && let Some(id) = id.get().checked_sub(1000000000000)
                    && let Some(id) = <NonZero<_>>::new(id)
                {
                    Ok(id)
                } else {
                    panic!("PeerId contains channel ID that fails smart constructors")
                }
            }
        }
    }
}

impl PeerAuth {
    /// Construct a new peer authentication using Telegram's `access_hash` value.
    pub fn from_hash(access_hash: i64) -> Self {
        PeerAuth(access_hash)
    }

    /// Grants access to the internal access hash.
    pub fn hash(&self) -> i64 {
        self.0
    }
}

impl Default for PeerAuth {
    /// Returns the ambient authority to authorize peers only when Telegram considers it valid.
    ///
    /// The internal representation uses `0` to signal the ambient authority,
    /// although this might happen to be the actual witness used by some peers.
    fn default() -> Self {
        Self(0)
    }
}

impl PeerInfo {
    /// Returns the `PeerId` represented by this info.
    ///
    /// The returned [`PeerId::kind()`] will never be [`PeerKind::UserSelf`].
    pub const fn id(&self) -> PeerId {
        match self {
            PeerInfo::User { id, .. } => unwrap_ok!(PeerId::user_checked(*id)),
            PeerInfo::Chat { id } => unwrap_ok!(PeerId::chat_checked(*id)),
            PeerInfo::Channel { id, .. } => unwrap_ok!(PeerId::channel_checked(*id)),
        }
    }

    /// Returns the `PeerAuth` stored in this info, or [`PeerAuth::default()`] if that info is not known.
    pub const fn auth(&self) -> PeerAuth {
        match self {
            PeerInfo::User {
                auth: Some(auth), ..
            } => *auth,
            PeerInfo::Channel {
                auth: Some(auth), ..
            } => *auth,
            PeerInfo::User { .. } | PeerInfo::Chat { .. } | PeerInfo::Channel { .. } => {
                AMBIENT_PEER_AUTH
            }
        }
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bot_api_dialog_id().fmt(f)
    }
}

impl From<PeerInfo> for PeerRef {
    fn from(peer: PeerInfo) -> Self {
        PeerRef {
            id: peer.id(),
            auth: peer.auth(),
        }
    }
}

impl From<tl::enums::Peer> for PeerId {
    fn from(peer: tl::enums::Peer) -> Self {
        match peer {
            tl::enums::Peer::User(user) => PeerId::from(user),
            tl::enums::Peer::Chat(chat) => PeerId::from(chat),
            tl::enums::Peer::Channel(channel) => PeerId::from(channel),
        }
    }
}

impl From<tl::types::PeerUser> for PeerId {
    fn from(user: tl::types::PeerUser) -> Self {
        PeerId::user(user.user_id)
    }
}

impl From<tl::types::PeerChat> for PeerId {
    fn from(chat: tl::types::PeerChat) -> Self {
        PeerId::chat(chat.chat_id)
    }
}

impl From<tl::types::PeerChannel> for PeerId {
    fn from(channel: tl::types::PeerChannel) -> Self {
        PeerId::channel(channel.channel_id)
    }
}

impl From<tl::enums::InputPeer> for PeerRef {
    fn from(peer: tl::enums::InputPeer) -> Self {
        match peer {
            tl::enums::InputPeer::Empty => {
                panic!("InputPeer::Empty cannot be converted to any Peer");
            }
            tl::enums::InputPeer::PeerSelf => PeerRef {
                id: SELF_USER_PEER_ID,
                auth: PeerAuth::default(),
            },
            tl::enums::InputPeer::User(user) => PeerRef::from(user),
            tl::enums::InputPeer::Chat(chat) => PeerRef::from(chat),
            tl::enums::InputPeer::Channel(channel) => PeerRef::from(channel),
            tl::enums::InputPeer::UserFromMessage(user) => PeerRef::from(*user),
            tl::enums::InputPeer::ChannelFromMessage(channel) => PeerRef::from(*channel),
        }
    }
}

impl From<tl::types::InputPeerSelf> for PeerRef {
    fn from(_: tl::types::InputPeerSelf) -> Self {
        PeerRef {
            id: SELF_USER_PEER_ID,
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::InputPeerUser> for PeerRef {
    fn from(user: tl::types::InputPeerUser) -> Self {
        PeerRef {
            id: PeerId::user(user.user_id),
            auth: PeerAuth::from_hash(user.access_hash),
        }
    }
}

impl From<tl::types::InputPeerChat> for PeerRef {
    fn from(chat: tl::types::InputPeerChat) -> Self {
        PeerRef {
            id: PeerId::chat(chat.chat_id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::InputPeerChannel> for PeerRef {
    fn from(channel: tl::types::InputPeerChannel) -> Self {
        PeerRef {
            id: PeerId::channel(channel.channel_id),
            auth: PeerAuth::from_hash(channel.access_hash),
        }
    }
}

impl From<tl::types::InputPeerUserFromMessage> for PeerRef {
    fn from(user: tl::types::InputPeerUserFromMessage) -> Self {
        // Not currently willing to make PeerRef significantly larger to accomodate for this uncommon type.
        PeerRef {
            id: PeerId::user(user.user_id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::InputPeerChannelFromMessage> for PeerRef {
    fn from(channel: tl::types::InputPeerChannelFromMessage) -> Self {
        // Not currently willing to make PeerRef significantly larger to accomodate for this uncommon type.
        PeerRef {
            id: PeerId::channel(channel.channel_id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::enums::User> for PeerRef {
    fn from(user: tl::enums::User) -> Self {
        match user {
            grammers_tl_types::enums::User::Empty(user) => PeerRef::from(user),
            grammers_tl_types::enums::User::User(user) => PeerRef::from(user),
        }
    }
}

impl From<tl::types::UserEmpty> for PeerRef {
    fn from(user: tl::types::UserEmpty) -> Self {
        PeerRef {
            id: PeerId::user(user.id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::User> for PeerRef {
    fn from(user: tl::types::User) -> Self {
        PeerRef {
            id: if user.is_self {
                PeerId::self_user()
            } else {
                PeerId::user(user.id)
            },
            auth: user
                .access_hash
                .map(PeerAuth::from_hash)
                .unwrap_or(PeerAuth::default()),
        }
    }
}

impl From<tl::enums::Chat> for PeerRef {
    fn from(chat: tl::enums::Chat) -> Self {
        match chat {
            grammers_tl_types::enums::Chat::Empty(chat) => PeerRef::from(chat),
            grammers_tl_types::enums::Chat::Chat(chat) => PeerRef::from(chat),
            grammers_tl_types::enums::Chat::Forbidden(chat) => PeerRef::from(chat),
            grammers_tl_types::enums::Chat::Channel(channel) => PeerRef::from(channel),
            grammers_tl_types::enums::Chat::ChannelForbidden(channel) => PeerRef::from(channel),
        }
    }
}

impl From<tl::types::ChatEmpty> for PeerRef {
    fn from(chat: tl::types::ChatEmpty) -> Self {
        PeerRef {
            id: PeerId::chat(chat.id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::Chat> for PeerRef {
    fn from(chat: tl::types::Chat) -> Self {
        PeerRef {
            id: PeerId::chat(chat.id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::ChatForbidden> for PeerRef {
    fn from(chat: tl::types::ChatForbidden) -> Self {
        PeerRef {
            id: PeerId::chat(chat.id),
            auth: PeerAuth::default(),
        }
    }
}

impl From<tl::types::Channel> for PeerRef {
    fn from(channel: tl::types::Channel) -> Self {
        PeerRef {
            id: PeerId::channel(channel.id),
            auth: channel
                .access_hash
                .map(PeerAuth::from_hash)
                .unwrap_or(PeerAuth::default()),
        }
    }
}

impl From<tl::types::ChannelForbidden> for PeerRef {
    fn from(channel: tl::types::ChannelForbidden) -> Self {
        PeerRef {
            id: PeerId::channel(channel.id),
            auth: PeerAuth::from_hash(channel.access_hash),
        }
    }
}

impl From<PeerId> for tl::enums::Peer {
    fn from(peer: PeerId) -> Self {
        let bare_id = peer.bare_id();
        match peer.kind() {
            PeerKind::User => tl::enums::Peer::User(tl::types::PeerUser {
                user_id: bare_id.expect("PeerKind::User").get(),
            }),
            PeerKind::UserSelf => tl::enums::Peer::User(tl::types::PeerUser {
                user_id: bare_id.expect("PeerKind::UserSelf").get(),
            }),
            PeerKind::Chat => tl::enums::Peer::Chat(tl::types::PeerChat {
                chat_id: bare_id.expect("PeerKind::Chat").get(),
            }),
            PeerKind::Channel => tl::enums::Peer::Channel(tl::types::PeerChannel {
                channel_id: bare_id.expect("PeerKind::Channel").get(),
            }),
        }
    }
}

impl From<PeerRef> for tl::enums::InputPeer {
    fn from(peer: PeerRef) -> Self {
        match peer.id.kind() {
            PeerKind::User => tl::enums::InputPeer::User(tl::types::InputPeerUser {
                user_id: peer.id.bare_id().expect("PeerKind::User").get(),
                access_hash: peer.auth.hash(),
            }),
            PeerKind::UserSelf => tl::enums::InputPeer::PeerSelf,
            PeerKind::Chat => tl::enums::InputPeer::Chat(tl::types::InputPeerChat {
                chat_id: peer.id.bare_id().expect("PeerKind::Chat").get(),
            }),
            PeerKind::Channel => tl::enums::InputPeer::Channel(tl::types::InputPeerChannel {
                channel_id: peer.id.bare_id().expect("PeerKind::Channel").get(),
                access_hash: peer.auth.hash(),
            }),
        }
    }
}

impl From<PeerRef> for tl::enums::InputUser {
    fn from(peer: PeerRef) -> Self {
        match peer.id.kind() {
            PeerKind::User => tl::enums::InputUser::User(tl::types::InputUser {
                user_id: peer.id.bare_id().expect("PeerKind::User").get(),
                access_hash: peer.auth.hash(),
            }),
            PeerKind::UserSelf => tl::enums::InputUser::UserSelf,
            PeerKind::Chat => tl::enums::InputUser::Empty,
            PeerKind::Channel => tl::enums::InputUser::Empty,
        }
    }
}

impl From<PeerRef> for NonZeroI64 {
    fn from(peer: PeerRef) -> Self {
        match peer.id.kind() {
            PeerKind::User => NON_ZERO_EMPTY_CHAT_ID,
            PeerKind::UserSelf => NON_ZERO_EMPTY_CHAT_ID,
            PeerKind::Chat => peer.id.bare_id().expect("PeerKind::Chat"),
            PeerKind::Channel => NON_ZERO_EMPTY_CHAT_ID,
        }
    }
}

impl From<PeerRef> for tl::enums::InputChannel {
    fn from(peer: PeerRef) -> Self {
        match peer.id.kind() {
            PeerKind::User => tl::enums::InputChannel::Empty,
            PeerKind::UserSelf => tl::enums::InputChannel::Empty,
            PeerKind::Chat => tl::enums::InputChannel::Empty,
            PeerKind::Channel => tl::enums::InputChannel::Channel(tl::types::InputChannel {
                channel_id: peer.id.bare_id().expect("PeerKind::Channel").get(),
                access_hash: peer.auth.hash(),
            }),
        }
    }
}

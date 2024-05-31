use crate::probe::Command;
use nostr_types::RelayMessage;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    CannotPost,
    ChannelIsClosed,
    General(String),
    Http(http::Error),
    Join(tokio::task::JoinError),
    Json(serde_json::Error),
    NostrTypes(nostr_types::Error),
    NotChallenged,
    Reqwest(reqwest::Error),
    SendCommand(tokio::sync::mpsc::error::SendError<Command>),
    SendRelayMessage(tokio::sync::mpsc::error::SendError<RelayMessage>),
    Timeout(tokio::time::error::Elapsed),
    Websocket(tungstenite::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::CannotPost => write!(f, "Cannot post benign event as registered user"),
            Error::ChannelIsClosed => write!(f, "Channel is closed"),
            Error::General(s) => write!(f, "General: {s}"),
            Error::Http(e) => write!(f, "Http: {e}"),
            Error::Join(e) => write!(f, "Tokio join: {e}"),
            Error::Json(e) => write!(f, "JSON: {e}"),
            Error::NostrTypes(e) => write!(f, "nostr-types: {e}"),
            Error::NotChallenged => write!(f, "Not challenged for AUTH"),
            Error::Reqwest(e) => write!(f, "Http: {e}"),
            Error::SendCommand(e) => write!(f, "Send Command: {e}"),
            Error::SendRelayMessage(e) => write!(f, "Send Relay Message: {e}"),
            Error::Timeout(e) => write!(f, "Timeout: {e}"),
            Error::Websocket(e) => write!(f, "Websocket: {e}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Http(inner) => Some(inner),
            Error::Join(inner) => Some(inner),
            Error::Json(inner) => Some(inner),
            Error::NostrTypes(inner) => Some(inner),
            Error::Reqwest(inner) => Some(inner),
            Error::SendCommand(inner) => Some(inner),
            Error::SendRelayMessage(inner) => Some(inner),
            Error::Timeout(inner) => Some(inner),
            Error::Websocket(inner) => Some(inner),
            _ => None,
        }
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Error {
        Error::Http(e)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Error {
        Error::Join(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(e)
    }
}

impl From<nostr_types::Error> for Error {
    fn from(e: nostr_types::Error) -> Error {
        Error::NostrTypes(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::Reqwest(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Command>> for Error {
    fn from(e: tokio::sync::mpsc::error::SendError<Command>) -> Error {
        Error::SendCommand(e)
    }
}

impl From<tokio::sync::mpsc::error::SendError<RelayMessage>> for Error {
    fn from(e: tokio::sync::mpsc::error::SendError<RelayMessage>) -> Error {
        Error::SendRelayMessage(e)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(e: tokio::time::error::Elapsed) -> Error {
        Error::Timeout(e)
    }
}

impl From<tungstenite::Error> for Error {
    fn from(e: tungstenite::Error) -> Error {
        Error::Websocket(e)
    }
}

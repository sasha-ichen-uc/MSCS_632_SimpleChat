use chrono::{DateTime, Local};
use std::fmt;

/// How a message is routed. This is the enum half of the message model —
/// Go represents the same idea with a recipient string plus a type constant,
/// since it has no tagged-union enums.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Direct { recipient: String },
    Broadcast,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Direct { recipient } => write!(f, "{}", recipient),
            MessageType::Broadcast => write!(f, "all"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub msg_type: MessageType,
    pub text: String,
    pub timestamp: DateTime<Local>,
}

impl Message {
    pub fn new(sender: impl Into<String>, msg_type: MessageType, text: impl Into<String>) -> Self {
        Message {
            sender: sender.into(),
            msg_type,
            text: text.into(),
            timestamp: Local::now(),
        }
    }

    /// Consistent display format shared by the handler, filter, and search output.
    pub fn format(&self) -> String {
        format!(
            "[{}] {} -> {}: {}",
            self.timestamp.format("%H:%M:%S%.3f"),
            self.sender,
            self.msg_type,
            self.text
        )
    }
}

/// Errors the chat app can hit. Every caller must match on this or propagate
/// it with `?` — the compiler won't let a `Result` be silently ignored the
/// way Go allows an unchecked `error` return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatError {
    UnknownUser(String),
    EmptyMessage,
    NoResults(String),
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatError::UnknownUser(user) => write!(f, "unknown user '{}'", user),
            ChatError::EmptyMessage => write!(f, "message text cannot be empty"),
            ChatError::NoResults(desc) => write!(f, "no results for {}", desc),
        }
    }
}

impl std::error::Error for ChatError {}

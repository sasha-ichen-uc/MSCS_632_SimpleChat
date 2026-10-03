use crate::message::{ChatError, Message};

/// Owns the message history. By design only the handler task in `chat.rs`
/// ever touches a `MessageStore` — everyone else talks to it through a
/// channel, so there is no shared mutable state for the borrow checker (or
/// us) to worry about.
pub struct MessageStore {
    messages: Vec<Message>,
}

impl MessageStore {
    pub fn new() -> Self {
        MessageStore {
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Messages sent BY the given user. `Result` instead of a plain `Vec`
    /// because an empty match is treated as a reportable condition, not a
    /// silent empty list.
    pub fn filter_by_user(&self, user: &str) -> Result<Vec<&Message>, ChatError> {
        let matches: Vec<&Message> = self
            .messages
            .iter()
            .filter(|m| m.sender == user)
            .collect();

        if matches.is_empty() {
            Err(ChatError::NoResults(format!("messages from '{}'", user)))
        } else {
            Ok(matches)
        }
    }

    /// Case-insensitive substring search over message text.
    pub fn search_by_keyword(&self, keyword: &str) -> Result<Vec<&Message>, ChatError> {
        let needle = keyword.to_lowercase();
        let matches: Vec<&Message> = self
            .messages
            .iter()
            .filter(|m| m.text.to_lowercase().contains(&needle))
            .collect();

        if matches.is_empty() {
            Err(ChatError::NoResults(format!("keyword '{}'", keyword)))
        } else {
            Ok(matches)
        }
    }
}

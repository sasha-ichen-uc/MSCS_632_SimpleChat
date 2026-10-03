use std::collections::HashSet;

use tokio::sync::{mpsc, oneshot};

use crate::message::{ChatError, Message, MessageType};
use crate::store::MessageStore;

/// Requests the simulated users (and `main`) can send to the handler task.
/// The handler is the single owner of the `MessageStore`; everything else
/// only ever sees a `Sender<ChatCommand>`.
pub enum ChatCommand {
    Send {
        message: Message,
        respond_to: oneshot::Sender<Result<(), ChatError>>,
    },
    FilterByUser {
        user: String,
        respond_to: oneshot::Sender<Result<Vec<String>, ChatError>>,
    },
    SearchKeyword {
        keyword: String,
        respond_to: oneshot::Sender<Result<Vec<String>, ChatError>>,
    },
    Shutdown,
}

/// Runs the central message handler. Owns the `MessageStore` and the set of
/// known users for the lifetime of the chat session, and exits when the
/// channel closes or a `Shutdown` command arrives.
pub async fn run_handler(mut rx: mpsc::Receiver<ChatCommand>, known_users: HashSet<String>) {
    let mut store = MessageStore::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            ChatCommand::Send {
                message,
                respond_to,
            } => {
                let result = validate_and_store(&mut store, &known_users, message);
                let _ = respond_to.send(result);
            }
            ChatCommand::FilterByUser { user, respond_to } => {
                let result = store
                    .filter_by_user(&user)
                    .map(|msgs| msgs.iter().map(|m| m.format()).collect());
                let _ = respond_to.send(result);
            }
            ChatCommand::SearchKeyword {
                keyword,
                respond_to,
            } => {
                let result = store
                    .search_by_keyword(&keyword)
                    .map(|msgs| msgs.iter().map(|m| m.format()).collect());
                let _ = respond_to.send(result);
            }
            ChatCommand::Shutdown => break,
        }
    }
}

fn validate_and_store(
    store: &mut MessageStore,
    known_users: &HashSet<String>,
    message: Message,
) -> Result<(), ChatError> {
    if message.text.trim().is_empty() {
        return Err(ChatError::EmptyMessage);
    }
    if let MessageType::Direct { recipient } = &message.msg_type {
        if !known_users.contains(recipient) {
            return Err(ChatError::UnknownUser(recipient.clone()));
        }
    }

    println!("{}", message.format());
    store.add(message);
    Ok(())
}

/// Simulates one user sending one message: builds it and sends it to the
/// handler over the channel, then prints the outcome. Several of these run
/// concurrently as separate tokio tasks.
pub async fn send_as_user(
    tx: mpsc::Sender<ChatCommand>,
    sender: &str,
    msg_type: MessageType,
    text: &str,
) {
    let message = Message::new(sender, msg_type, text);
    let (resp_tx, resp_rx) = oneshot::channel();

    if tx
        .send(ChatCommand::Send {
            message,
            respond_to: resp_tx,
        })
        .await
        .is_err()
    {
        eprintln!("Error: chat handler is not running");
        return;
    }

    match resp_rx.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => println!("Error: {} ({} tried to send)", e, sender),
        Err(_) => eprintln!("Error: no response from chat handler"),
    }
}

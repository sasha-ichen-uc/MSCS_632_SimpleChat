mod chat;
mod message;
mod store;

use std::collections::HashSet;

use tokio::sync::{mpsc, oneshot};

use chat::{run_handler, send_as_user, ChatCommand};
use message::MessageType;

#[tokio::main]
async fn main() {
    let known_users: HashSet<String> = ["alice", "bob", "carol"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let (tx, rx) = mpsc::channel(32);
    let handler = tokio::spawn(run_handler(rx, known_users));

    println!("-- sending messages --");

    // Several simulated users send concurrently, including two error cases:
    // an unknown recipient and an empty message.
    let sends = vec![
        tokio::spawn(send_as_user(
            tx.clone(),
            "alice",
            MessageType::Direct {
                recipient: "bob".to_string(),
            },
            "hey bob, are we still on for rust tonight?",
        )),
        tokio::spawn(send_as_user(
            tx.clone(),
            "bob",
            MessageType::Direct {
                recipient: "alice".to_string(),
            },
            "yep, see you at 7",
        )),
        tokio::spawn(send_as_user(
            tx.clone(),
            "carol",
            MessageType::Broadcast,
            "reminder: demo freeze is tonight",
        )),
        tokio::spawn(send_as_user(
            tx.clone(),
            "alice",
            MessageType::Direct {
                recipient: "dave".to_string(), // unknown user -> expect an error
            },
            "is dave around?",
        )),
        tokio::spawn(send_as_user(
            tx.clone(),
            "bob",
            MessageType::Broadcast,
            "", // empty message -> expect an error
        )),
    ];

    for send in sends {
        let _ = send.await;
    }

    println!("\n-- messages from alice --");
    print_filter_by_user(&tx, "alice").await;

    println!("\n-- messages from an unknown user --");
    print_filter_by_user(&tx, "dave").await;

    println!("\n-- keyword search: 'rust' --");
    print_search_keyword(&tx, "rust").await;

    println!("\n-- keyword search with no matches: 'golang' --");
    print_search_keyword(&tx, "golang").await;

    let _ = tx.send(ChatCommand::Shutdown).await;
    drop(tx);
    let _ = handler.await;
}

async fn print_filter_by_user(tx: &mpsc::Sender<ChatCommand>, user: &str) {
    let (resp_tx, resp_rx) = oneshot::channel();
    let _ = tx
        .send(ChatCommand::FilterByUser {
            user: user.to_string(),
            respond_to: resp_tx,
        })
        .await;
    print_lines(resp_rx.await);
}

async fn print_search_keyword(tx: &mpsc::Sender<ChatCommand>, keyword: &str) {
    let (resp_tx, resp_rx) = oneshot::channel();
    let _ = tx
        .send(ChatCommand::SearchKeyword {
            keyword: keyword.to_string(),
            respond_to: resp_tx,
        })
        .await;
    print_lines(resp_rx.await);
}

fn print_lines(result: Result<Result<Vec<String>, message::ChatError>, oneshot::error::RecvError>) {
    match result {
        Ok(Ok(lines)) => {
            for line in lines {
                println!("{}", line);
            }
        }
        Ok(Err(e)) => println!("Error: {}", e),
        Err(_) => println!("Error: no response from chat handler"),
    }
}

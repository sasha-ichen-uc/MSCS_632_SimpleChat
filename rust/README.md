# Simple Chat Application — Rust

Rust implementation for MSCS-632 Deliverable: simulated multi-user chat, no
network and no GUI. See the project-level planning doc for the shared spec.

## Features

- Simulated users (`alice`, `bob`, `carol`) created at startup.
- Users send a message to one other user (`MessageType::Direct`) or to
  everyone (`MessageType::Broadcast`); several users send concurrently as
  tokio tasks.
- Every message is timestamped and stored in memory with sender, recipient,
  and text.
- Filter history by sender.
- Case-insensitive keyword search over message text.
- Errors (unknown recipient, empty message, no search/filter results) print
  a clear message instead of panicking.

## Structure

- `src/message.rs` — `Message` struct, `MessageType` enum, `ChatError` enum.
- `src/store.rs` — `MessageStore`: history, filter, search.
- `src/chat.rs` — the async handler task and channel-based commands.
- `src/main.rs` — demo scenario.

## Design

A single handler task owns the `MessageStore`. Every simulated user task
only holds a `Sender<ChatCommand>` and talks to the handler over a `tokio`
mpsc channel, using a `oneshot` channel per request to get a response back.
This keeps the history behind one owner, so the borrow checker never has to
reason about shared mutable state across tasks.

## Build & run

Requires Rust (stable) via [rustup](https://rustup.rs/).

```sh
cargo run
```

Run the test-free demo scenario above; it sends several messages
concurrently (including two deliberately invalid ones), then filters by
user and searches by keyword, including a no-match case.

## Dependencies

- `tokio` (async runtime, channels) — Go gets goroutines/channels for free;
  Rust needs this crate.
- `chrono` (timestamps) — same story, Go's `time` package is built in.

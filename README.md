# EaseChat

Simple message exchange network with channel-subscription based structure. 
EaseChat was originally designed as a chat message exchanging service for games, but its usage could be widen to debug messages
and commands for maintaining.

Although the first version of EaseChat Nexus may be less flexible, we deployed it on production use months ago.
Its record on non-pausing and reboot-free continuous execution is 3,432 hours on our mini-game server network region, since 30 Jan to 22 Jun 2019, killed mistaken by a ctrl-c quit. The network consists of ~350 endpoints and produces ~1k messages on average per second, their CPU and RAM usage being low at the same time.
Players mostly suggest it 'Great' in our recent survey on this part of the game's experience.

## Project structure

| Name | Description |
|:----|:-----------|
| easechat-client-j | Netty-based Java implementation of EaseChat client oriented to production |
| easechat-nexus-rs | Production Rust EaseChat Nexus implementation using Tokio and tokio-tungstenite |
| easechat-exec-rs | Legacy in-progress Rust CLI application, excluded from the active workspace |
| easechat-nexus-v2-rs | Legacy in-progress redesigned Rust protocol library, excluded from the active workspace |
| easechat-record-rs | Legacy in-progress Rust recorder client, excluded from the active workspace |

## Run EaseChat Nexus

1. Install [rust](https://rust-lang.org/), or run `rustup update` if already installed.
2. Execute `cargo run -p ease_chat_nexus` in project root.
3. The default listen address is `0.0.0.0:6500`. Pass the first CLI argument to override it, for example:

```bash
cargo run -p ease_chat_nexus -- 127.0.0.1:6500
```

For production builds:

```bash
cargo build --release -p ease_chat_nexus
./target/release/ease_chat_nexus 0.0.0.0:6500
```

The Dockerfile in the parent repository builds the release binary in a Rust builder image and runs only the compiled Nexus executable in the runtime image.

## Protocol compatibility

The production Nexus keeps the original WebSocket text protocol unchanged:

- `1h` client hello
- `1c` channel subscription
- `1t` message transmit
- `1d` client disconnect
- `1r` server receive message

Protocol string length fields are interpreted as UTF-8 byte lengths, matching the Java client protocol encoding. Messages may contain `|` inside length-delimited fields.

## Console commands

The server keeps the original console command set:

- `list` / `l`
- `push` / `msg` / `say` / `send` / `p`
- `stop` / `quit` / `exit` / `q`
- `huaji`

## Stress testing

Build the release server and stress client:

```bash
cargo build --release -p ease_chat_nexus --example stress
```

Start Nexus:

```bash
./target/release/ease_chat_nexus 127.0.0.1:16500
```

Run a baseline test with 350 clients and a 10-channel fanout:

```bash
./target/release/examples/stress \
  --url ws://127.0.0.1:16500 \
  --clients 350 \
  --publishers 10 \
  --channels 10 \
  --duration 30 \
  --message-size 64 \
  --target-rate 3500 \
  --connect-parallel 128
```

Run a high-fanout broadcast test:

```bash
./target/release/examples/stress \
  --url ws://127.0.0.1:16500 \
  --clients 350 \
  --publishers 10 \
  --channels 1 \
  --duration 30 \
  --message-size 64 \
  --target-rate 400 \
  --connect-parallel 128
```

Current local stress results on a Windows development machine show a stable range around 120k-140k client deliveries per second. The exact limit depends on channel fanout, client read speed, and the machine running both Nexus and stress clients.

## Protocol specification

[中文协议文档](https://github.com/EaseCation/ease_chat/blob/master/protocol-zh.md)

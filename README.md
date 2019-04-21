# EaseChat

Simple message exchange network with channel-subscription based structure. 
EaseChat was originally designed as a chat message exchanging service for games, but its usage could be widen to debug messages
and commands for maintaining.

Although the first version of EaseChat Nexus may be outdated, but we deployed it on production use months ago. 
It continues to exchange massive messages for our game server for 3,000 hours without rebooting and pausing, its CPU and RAM usage
being low at the same time. Players mostly suggest it 'Great' in our recent survey on this part of the game's experience.

## Project structure

| Name | Description |
|:----|:-----------|
| easechat-client-j | Netty-based Java implementation of EaseChat client oriented to production |
| easechat-exec-rs | (In progress) Complete cli application of EaseChat in Rust |
| easechat-nexus-rs | First version of Rust EaseChat nexus impl using mio and ws |
| easechat-nexus-v2-rs | (In progress) Redesigned Rust EaseChat protocol library intended for general use |
| easechat-record-rs | (In progress) Simple Rust cli EaseChat client app, connect to nexus and save message via SQLite | 

## Run EaseChat Nexus

1. Install [rust](https://rust-lang.org/), or run `rustup update` if already installed.
2. Execute `cargo run -p ease_chat_nexus` in project root.

Step 1 is necessary because this EaseChat Nexus implementation requires rust edition 2018 installed. 
This implementation is written in rust version 1.32.0-stable.

## Protocol specification

[中文协议文档](https://github.com/EaseCation/ease_chat/blob/master/protocol-zh.md)

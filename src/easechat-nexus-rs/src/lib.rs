use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::time;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message as WsMessage};
use tracing::{debug, info, warn};

pub const DEFAULT_ADDR: &str = "0.0.0.0:6500";
pub const OUTBOUND_QUEUE_SIZE: usize = 16;
const SERVICE_CHANNEL_SIZE: usize = 4096;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

pub type ServiceSender = mpsc::Sender<ServiceEvent>;
pub type OutboundSender = mpsc::Sender<OutboundMessage>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMessage {
    Hello {
        ep_id: String,
    },
    Subscribe {
        channel: String,
        valid_secs: u64,
        valid_nanos: u64,
    },
    Transmit {
        channel: String,
        text: String,
    },
    Disconnect {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleCommand {
    Status {
        keyword: Option<String>,
    },
    Push {
        source: String,
        channel: String,
        text: String,
    },
    Shutdown {
        reason: String,
    },
    Huaji,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseKind {
    Normal,
    Protocol,
    Unsupported,
    Invalid,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboundMessage {
    Text(String),
    Pong(Vec<u8>),
    Close { kind: CloseKind, reason: String },
}

#[derive(Debug)]
pub enum ServiceEvent {
    Connected {
        connection_id: u64,
        peer: Option<SocketAddr>,
        sender: OutboundSender,
    },
    Inbound {
        connection_id: u64,
        message: ClientMessage,
    },
    Disconnected {
        connection_id: u64,
        reason: String,
    },
    Console(ConsoleCommand),
    CleanupExpired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    pub kind: CloseKind,
    pub reason: &'static str,
}

impl ProtocolError {
    fn new(kind: CloseKind, reason: &'static str) -> Self {
        Self { kind, reason }
    }
}

#[derive(Debug, Clone)]
struct ConnectionState {
    ep_id: Option<String>,
    peer: Option<SocketAddr>,
    sender: OutboundSender,
}

#[derive(Debug, Clone)]
struct EndpointState {
    connection_id: u64,
    sender: OutboundSender,
}

#[derive(Debug, Clone)]
struct SubscriptionState {
    connection_id: u64,
    expires_at: Instant,
}

#[derive(Debug, Default)]
pub struct ServiceState {
    connections: HashMap<u64, ConnectionState>,
    endpoints: HashMap<String, EndpointState>,
    subscriptions: HashMap<String, HashMap<String, SubscriptionState>>,
}

impl ServiceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    pub fn channel_count(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn handle_event(&mut self, event: ServiceEvent) -> bool {
        match event {
            ServiceEvent::Connected {
                connection_id,
                peer,
                sender,
            } => {
                self.connections.insert(
                    connection_id,
                    ConnectionState {
                        ep_id: None,
                        peer,
                        sender,
                    },
                );
                if let Some(peer) = peer {
                    info!("[Addr {}] Connection open!", peer);
                } else {
                    info!("[Addr unknown] Connection open!");
                }
                true
            }
            ServiceEvent::Inbound {
                connection_id,
                message,
            } => self.handle_inbound(connection_id, message),
            ServiceEvent::Disconnected {
                connection_id,
                reason,
            } => {
                self.unregister_connection(connection_id, reason);
                true
            }
            ServiceEvent::Console(command) => self.handle_console(command),
            ServiceEvent::CleanupExpired => {
                self.cleanup_expired();
                true
            }
        }
    }

    fn handle_inbound(&mut self, connection_id: u64, message: ClientMessage) -> bool {
        match message {
            ClientMessage::Hello { ep_id } => {
                self.register_endpoint(connection_id, ep_id);
                true
            }
            ClientMessage::Subscribe {
                channel,
                valid_secs,
                valid_nanos,
            } => {
                if let Some(ep_id) = self.current_ep_id(connection_id) {
                    let now = Instant::now();
                    let duration = Duration::from_secs(valid_secs)
                        .saturating_add(Duration::from_nanos(valid_nanos));
                    let expires_at = now
                        .checked_add(duration)
                        .unwrap_or_else(|| now + Duration::from_secs(100 * 365 * 24 * 60 * 60));
                    self.subscriptions
                        .entry(channel.clone())
                        .or_default()
                        .insert(
                            ep_id.clone(),
                            SubscriptionState {
                                connection_id,
                                expires_at,
                            },
                        );
                    info!(
                        "[EpID {}] Registered new channel [{}], expire at {:?}",
                        ep_id, channel, expires_at
                    );
                    true
                } else {
                    self.close_connection(
                        connection_id,
                        CloseKind::Error,
                        "Connection unidentified".to_string(),
                    );
                    true
                }
            }
            ClientMessage::Transmit { channel, text } => {
                if let Some(ep_id) = self.current_ep_id(connection_id) {
                    let sent = self.push_message(&ep_id, &channel, &text);
                    debug!(
                        "[EpID {}] Sent to {} [{} client(s) excluding self]: {}",
                        ep_id, channel, sent, text
                    );
                    true
                } else {
                    self.close_connection(
                        connection_id,
                        CloseKind::Error,
                        "Connection unidentified".to_string(),
                    );
                    true
                }
            }
            ClientMessage::Disconnect { reason } => {
                self.close_connection(connection_id, CloseKind::Normal, reason);
                true
            }
        }
    }

    fn handle_console(&mut self, command: ConsoleCommand) -> bool {
        match command {
            ConsoleCommand::Status { keyword } => {
                self.print_status(keyword);
                true
            }
            ConsoleCommand::Push {
                source,
                channel,
                text,
            } => {
                let sent = self.push_message(&source, &channel, &text);
                info!(
                    "[EpID {}] Sent to {} [{} client(s) excluding self]: {}",
                    source, channel, sent, text
                );
                true
            }
            ConsoleCommand::Shutdown { reason } => {
                info!("Shutting down with reason [{}]...", reason);
                let connection_ids: Vec<u64> = self.connections.keys().copied().collect();
                for connection_id in connection_ids {
                    self.close_connection(connection_id, CloseKind::Normal, reason.clone());
                }
                self.connections.clear();
                self.endpoints.clear();
                self.subscriptions.clear();
                false
            }
            ConsoleCommand::Huaji => {
                info!("[Module CMDLINE] huaji");
                true
            }
        }
    }

    fn register_endpoint(&mut self, connection_id: u64, ep_id: String) {
        let sender = if let Some(connection) = self.connections.get(&connection_id) {
            connection.sender.clone()
        } else {
            return;
        };

        if let Some(old_ep_id) = self
            .connections
            .get(&connection_id)
            .and_then(|connection| connection.ep_id.clone())
            .filter(|old_ep_id| old_ep_id != &ep_id)
        {
            self.remove_endpoint_if_connection_matches(&old_ep_id, connection_id);
        }

        if let Some(previous) = self.endpoints.get(&ep_id).cloned() {
            if previous.connection_id != connection_id {
                self.remove_endpoint_if_connection_matches(&ep_id, previous.connection_id);
                if let Some(connection) = self.connections.get_mut(&previous.connection_id) {
                    connection.ep_id = None;
                }
                let _ = previous.sender.try_send(OutboundMessage::Close {
                    kind: CloseKind::Error,
                    reason: "Duplicate endpoint identity".to_string(),
                });
            }
        }

        if let Some(connection) = self.connections.get_mut(&connection_id) {
            connection.ep_id = Some(ep_id.clone());
        }
        self.endpoints.insert(
            ep_id.clone(),
            EndpointState {
                connection_id,
                sender,
            },
        );
        info!("[EpID {}] Connection identified!", ep_id);
    }

    fn unregister_connection(&mut self, connection_id: u64, reason: String) {
        let state = self.connections.remove(&connection_id);
        if let Some(connection) = state {
            if let Some(ep_id) = connection.ep_id {
                self.remove_endpoint_if_connection_matches(&ep_id, connection_id);
                info!("[EpID {}] Connection closed, Reason:[{}]", ep_id, reason);
            } else if let Some(peer) = connection.peer {
                info!("[Addr {}] Connection closed, Reason:[{}]", peer, reason);
            } else {
                info!("[Unidentified] Connection closed, Reason:[{}]", reason);
            }
        }
    }

    fn remove_endpoint_if_connection_matches(&mut self, ep_id: &str, connection_id: u64) {
        let should_remove = self
            .endpoints
            .get(ep_id)
            .is_some_and(|endpoint| endpoint.connection_id == connection_id);
        if !should_remove {
            return;
        }
        self.endpoints.remove(ep_id);
        for subscribers in self.subscriptions.values_mut() {
            subscribers.remove(ep_id);
        }
        self.subscriptions
            .retain(|_, subscribers| !subscribers.is_empty());
    }

    fn current_ep_id(&self, connection_id: u64) -> Option<String> {
        self.connections
            .get(&connection_id)
            .and_then(|connection| connection.ep_id.clone())
            .filter(|ep_id| {
                self.endpoints
                    .get(ep_id)
                    .is_some_and(|endpoint| endpoint.connection_id == connection_id)
            })
    }

    fn push_message(&mut self, source: &str, channel: &str, text: &str) -> usize {
        let now = Instant::now();
        let payload = encode_receive_message(source, channel, text);
        let mut targets = Vec::new();
        let mut stale_endpoints = Vec::new();

        if let Some(subscribers) = self.subscriptions.get_mut(channel) {
            subscribers.retain(|ep_id, subscription| {
                let live = subscription.expires_at >= now
                    && self.endpoints.get(ep_id).is_some_and(|endpoint| {
                        endpoint.connection_id == subscription.connection_id
                    });
                if !live {
                    stale_endpoints.push(ep_id.clone());
                }
                live
            });

            for (ep_id, subscription) in subscribers.iter() {
                if ep_id == source {
                    continue;
                }
                if let Some(endpoint) = self.endpoints.get(ep_id) {
                    if endpoint.connection_id == subscription.connection_id {
                        targets.push((ep_id.clone(), endpoint.sender.clone()));
                    }
                }
            }
        }

        if let Some(subscribers) = self.subscriptions.get(channel) {
            if subscribers.is_empty() {
                self.subscriptions.remove(channel);
            }
        }

        let mut sent = 0;
        let mut failed = Vec::new();
        for (ep_id, sender) in targets {
            match sender.try_send(OutboundMessage::Text(payload.clone())) {
                Ok(()) => sent += 1,
                Err(error) => {
                    warn!("Failed to enqueue message for [{}]: {}", ep_id, error);
                    failed.push(ep_id);
                }
            }
        }

        for ep_id in stale_endpoints.into_iter().chain(failed) {
            if let Some(endpoint) = self.endpoints.get(&ep_id).cloned() {
                self.close_connection(
                    endpoint.connection_id,
                    CloseKind::Error,
                    "Connection unavailable".to_string(),
                );
            }
        }
        sent
    }

    fn close_connection(&mut self, connection_id: u64, kind: CloseKind, reason: String) {
        if let Some(connection) = self.connections.get(&connection_id) {
            let _ = connection.sender.try_send(OutboundMessage::Close {
                kind,
                reason: reason.clone(),
            });
        }
        self.unregister_connection(connection_id, reason);
    }

    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let endpoints = &self.endpoints;
        for subscribers in self.subscriptions.values_mut() {
            subscribers.retain(|ep_id, subscription| {
                subscription.expires_at >= now
                    && endpoints.get(ep_id).is_some_and(|endpoint| {
                        endpoint.connection_id == subscription.connection_id
                    })
            });
        }
        self.subscriptions
            .retain(|_, subscribers| !subscribers.is_empty());
    }

    fn print_status(&self, keyword: Option<String>) {
        if let Some(keyword) = keyword {
            let now = Instant::now();
            let mut channels = Vec::new();
            let mut subscribers = Vec::new();

            if let Some(subscriber_map) = self.subscriptions.get(&keyword) {
                for (ep_id, subscription) in subscriber_map {
                    subscribers.push((ep_id.clone(), subscription.expires_at));
                }
            }
            for (channel, subscriber_map) in &self.subscriptions {
                if let Some(subscription) = subscriber_map.get(&keyword) {
                    channels.push((channel.clone(), subscription.expires_at));
                }
            }

            if channels.is_empty() && subscribers.is_empty() {
                info!("[Module STATUS] No status found for keyword [{}].", keyword);
                return;
            }
            if !channels.is_empty() {
                let mut output =
                    format!("EpID [{}] listens {} channel(s):", keyword, channels.len());
                for (channel, expires_at) in channels {
                    output.push_str(&format!(
                        "\n\tchannel: [{}], {}",
                        channel,
                        format_expire(expires_at, now)
                    ));
                }
                info!("[Module STATUS] {}", output);
            }
            if !subscribers.is_empty() {
                let mut output = format!(
                    "Channel [{}] is subscribed by {} client(s):",
                    keyword,
                    subscribers.len()
                );
                for (ep_id, expires_at) in subscribers {
                    output.push_str(&format!(
                        "\n\tsender: [{}], {}",
                        ep_id,
                        format_expire(expires_at, now)
                    ));
                }
                info!("[Module STATUS] {}", output);
            }
        } else {
            info!(
                "[Module STATUS] {} clients identified, {} channels active.",
                self.endpoints.len(),
                self.subscriptions.len()
            );
        }
    }
}

fn format_expire(expires_at: Instant, now: Instant) -> String {
    if expires_at > now {
        format!("expire at {:?}", expires_at - now)
    } else {
        "already expired".to_string()
    }
}

pub fn service_channel() -> (ServiceSender, mpsc::Receiver<ServiceEvent>) {
    mpsc::channel(SERVICE_CHANNEL_SIZE)
}

pub async fn run_service(
    mut receiver: mpsc::Receiver<ServiceEvent>,
    shutdown_tx: watch::Sender<bool>,
) {
    info!("[Module MSG-SERV] Started!");
    let mut state = ServiceState::new();
    let mut cleanup = time::interval(CLEANUP_INTERVAL);

    loop {
        tokio::select! {
            Some(event) = receiver.recv() => {
                if !state.handle_event(event) {
                    let _ = shutdown_tx.send(true);
                    break;
                }
            }
            _ = cleanup.tick() => {
                state.handle_event(ServiceEvent::CleanupExpired);
            }
            else => break,
        }
    }
}

pub async fn run_accept_loop(
    listener: TcpListener,
    event_tx: ServiceSender,
    mut shutdown_rx: watch::Receiver<bool>,
) -> std::io::Result<()> {
    let mut next_connection_id = 1_u64;

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, peer) = accepted?;
                let connection_id = next_connection_id;
                next_connection_id = next_connection_id.wrapping_add(1).max(1);
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    handle_connection(connection_id, stream, Some(peer), tx).await;
                });
            }
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    break;
                }
                if changed.is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_connection(
    connection_id: u64,
    stream: TcpStream,
    peer: Option<SocketAddr>,
    event_tx: ServiceSender,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(stream) => stream,
        Err(error) => {
            warn!("WebSocket handshake failed for {:?}: {}", peer, error);
            return;
        }
    };
    let (mut writer, mut reader) = ws_stream.split();
    let (out_tx, mut out_rx) = mpsc::channel(OUTBOUND_QUEUE_SIZE);

    if event_tx
        .send(ServiceEvent::Connected {
            connection_id,
            peer,
            sender: out_tx.clone(),
        })
        .await
        .is_err()
    {
        return;
    }

    let writer_task = tokio::spawn(async move {
        while let Some(message) = out_rx.recv().await {
            let result = match message {
                OutboundMessage::Text(text) => writer.send(WsMessage::Text(text.into())).await,
                OutboundMessage::Pong(data) => writer.send(WsMessage::Pong(data.into())).await,
                OutboundMessage::Close { kind, reason } => {
                    let frame = CloseFrame {
                        code: close_code(kind),
                        reason: reason.into(),
                    };
                    let result = writer.send(WsMessage::Close(Some(frame))).await;
                    if let Err(error) = result {
                        debug!("WebSocket close failed: {}", error);
                    }
                    break;
                }
            };
            if let Err(error) = result {
                debug!("WebSocket write failed: {}", error);
                break;
            }
        }
        let _ = writer.close().await;
    });

    let mut close_reason = "socket closed".to_string();
    while let Some(message) = reader.next().await {
        match message {
            Ok(WsMessage::Text(text)) => match parse_client_message(text.as_str()) {
                Ok(client_message) => {
                    if event_tx
                        .send(ServiceEvent::Inbound {
                            connection_id,
                            message: client_message,
                        })
                        .await
                        .is_err()
                    {
                        close_reason = "service unavailable".to_string();
                        break;
                    }
                }
                Err(error) => {
                    let _ = out_tx.try_send(OutboundMessage::Close {
                        kind: error.kind,
                        reason: error.reason.to_string(),
                    });
                    close_reason = error.reason.to_string();
                    break;
                }
            },
            Ok(WsMessage::Binary(_)) => {
                let reason = "Please input as string text";
                let _ = out_tx.try_send(OutboundMessage::Close {
                    kind: CloseKind::Unsupported,
                    reason: reason.to_string(),
                });
                close_reason = reason.to_string();
                break;
            }
            Ok(WsMessage::Ping(data)) => {
                let _ = out_tx.try_send(OutboundMessage::Pong(data.to_vec()));
            }
            Ok(WsMessage::Pong(_)) => {}
            Ok(WsMessage::Close(frame)) => {
                close_reason = frame
                    .map(|frame| frame.reason.to_string())
                    .filter(|reason| !reason.is_empty())
                    .unwrap_or_else(|| "client closed".to_string());
                break;
            }
            Ok(WsMessage::Frame(_)) => {}
            Err(error) => {
                close_reason = error.to_string();
                break;
            }
        }
    }

    let _ = event_tx
        .send(ServiceEvent::Disconnected {
            connection_id,
            reason: close_reason,
        })
        .await;
    drop(out_tx);
    let _ = writer_task.await;
}

pub fn close_code(kind: CloseKind) -> CloseCode {
    match kind {
        CloseKind::Normal => CloseCode::Normal,
        CloseKind::Protocol => CloseCode::Protocol,
        CloseKind::Unsupported => CloseCode::Unsupported,
        CloseKind::Invalid => CloseCode::Invalid,
        CloseKind::Error => CloseCode::Error,
    }
}

pub fn parse_client_message(source: &str) -> Result<ClientMessage, ProtocolError> {
    let bytes = source.as_bytes();
    if bytes.first().copied() != Some(b'1') {
        return Err(ProtocolError::new(
            CloseKind::Protocol,
            "Protocol other than '1' is not supported",
        ));
    }
    let message_type = bytes.get(1).copied();
    if bytes.get(2).copied() != Some(b'|') {
        return Err(ProtocolError::new(
            CloseKind::Invalid,
            "Invalid message type: expected 'h', 't', 'c' or 'd'",
        ));
    }

    let mut cursor = 3;
    match message_type {
        Some(b'h') => Ok(ClientMessage::Hello {
            ep_id: read_protocol_string(source, &mut cursor)?,
        }),
        Some(b'c') => Ok(ClientMessage::Subscribe {
            channel: read_protocol_string(source, &mut cursor)?,
            valid_secs: read_protocol_number(source, &mut cursor)?,
            valid_nanos: read_protocol_number(source, &mut cursor)?,
        }),
        Some(b't') => Ok(ClientMessage::Transmit {
            channel: read_protocol_string(source, &mut cursor)?,
            text: read_protocol_string(source, &mut cursor)?,
        }),
        Some(b'd') => Ok(ClientMessage::Disconnect {
            reason: read_protocol_string(source, &mut cursor)?,
        }),
        _ => Err(ProtocolError::new(
            CloseKind::Invalid,
            "Invalid message type: expected 'h', 't', 'c' or 'd'",
        )),
    }
}

fn read_protocol_number(source: &str, cursor: &mut usize) -> Result<u64, ProtocolError> {
    let bytes = source.as_bytes();
    while *cursor < bytes.len() && !bytes[*cursor].is_ascii_digit() {
        *cursor += 1;
    }
    if *cursor >= bytes.len() {
        return Err(ProtocolError::new(CloseKind::Protocol, "Missing number"));
    }

    let mut value = 0_u64;
    let mut has_digit = false;
    while *cursor < bytes.len() && bytes[*cursor].is_ascii_digit() {
        has_digit = true;
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(bytes[*cursor] - b'0')))
            .ok_or_else(|| ProtocolError::new(CloseKind::Protocol, "Number too large"))?;
        *cursor += 1;
    }

    if !has_digit {
        return Err(ProtocolError::new(CloseKind::Protocol, "Missing number"));
    }
    if *cursor < bytes.len() && !bytes[*cursor].is_ascii_digit() {
        *cursor += 1;
    }
    Ok(value)
}

fn read_protocol_string(source: &str, cursor: &mut usize) -> Result<String, ProtocolError> {
    let len = read_protocol_number(source, cursor)? as usize;
    let end = cursor
        .checked_add(len)
        .ok_or_else(|| ProtocolError::new(CloseKind::Protocol, "Invalid string length"))?;
    if end > source.len() || !source.is_char_boundary(*cursor) || !source.is_char_boundary(end) {
        return Err(ProtocolError::new(
            CloseKind::Protocol,
            "Invalid string length",
        ));
    }
    let value = source[*cursor..end].to_string();
    *cursor = end;
    Ok(value)
}

pub fn encode_receive_message(source: &str, channel: &str, text: &str) -> String {
    format!(
        "1r|{}|{}|{}|{}|{}|{}",
        source.len(),
        source,
        channel.len(),
        channel,
        text.len(),
        text
    )
}

pub fn parse_console_command(line: &str) -> Result<Option<ConsoleCommand>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let (head, rest) = split_first_token(trimmed).ok_or_else(|| "Invalid command".to_string())?;
    match head {
        "quit" | "exit" | "stop" | "q" => Ok(Some(ConsoleCommand::Shutdown {
            reason: if rest.trim().is_empty() {
                "server shutdown".to_string()
            } else {
                rest.trim().to_string()
            },
        })),
        "list" | "l" => {
            let keyword = rest.trim();
            Ok(Some(ConsoleCommand::Status {
                keyword: (!keyword.is_empty()).then(|| keyword.to_string()),
            }))
        }
        "push" | "msg" | "say" | "send" | "p" => {
            let (source, rest) =
                split_first_token(rest.trim()).ok_or_else(|| "Missing sender id".to_string())?;
            let (channel, rest) =
                split_first_token(rest.trim()).ok_or_else(|| "Missing channel id".to_string())?;
            let text = rest.trim_start();
            if text.is_empty() {
                return Err("Missing message".to_string());
            }
            Ok(Some(ConsoleCommand::Push {
                source: source.to_string(),
                channel: channel.to_string(),
                text: text.to_string(),
            }))
        }
        "huaji" => Ok(Some(ConsoleCommand::Huaji)),
        _ => Err(format!("Unknown command: {}", head)),
    }
}

fn split_first_token(source: &str) -> Option<(&str, &str)> {
    let trimmed = source.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let end = trimmed
        .char_indices()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
        .unwrap_or(trimmed.len());
    Some((&trimmed[..end], &trimmed[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio::time::timeout;
    use tokio_tungstenite::connect_async;

    #[test]
    fn parse_text_protocol_with_pipe_and_utf8() {
        let message = parse_client_message("1t|8|c/中文|16|hello|世界🙂").unwrap();
        assert_eq!(
            message,
            ClientMessage::Transmit {
                channel: "c/中文".to_string(),
                text: "hello|世界🙂".to_string()
            }
        );
    }

    #[test]
    fn parse_supported_message_types() {
        assert_eq!(
            parse_client_message("1h|3|abc").unwrap(),
            ClientMessage::Hello {
                ep_id: "abc".to_string()
            }
        );
        assert_eq!(
            parse_client_message("1c|7|c/lobby|300|0").unwrap(),
            ClientMessage::Subscribe {
                channel: "c/lobby".to_string(),
                valid_secs: 300,
                valid_nanos: 0
            }
        );
        assert_eq!(
            parse_client_message("1d|11|just logout").unwrap(),
            ClientMessage::Disconnect {
                reason: "just logout".to_string()
            }
        );
    }

    #[test]
    fn reject_invalid_protocol() {
        assert_eq!(
            parse_client_message("2h|3|abc").unwrap_err().kind,
            CloseKind::Protocol
        );
        assert_eq!(
            parse_client_message("1x|3|abc").unwrap_err().kind,
            CloseKind::Invalid
        );
        assert_eq!(
            parse_client_message("1h|4|中").unwrap_err().kind,
            CloseKind::Protocol
        );
    }

    #[test]
    fn encode_receive_uses_utf8_byte_lengths() {
        assert_eq!(
            encode_receive_message("源", "c/中文", "hello|世界🙂"),
            "1r|3|源|8|c/中文|16|hello|世界🙂"
        );
    }

    #[test]
    fn parse_console_commands() {
        assert_eq!(
            parse_console_command("push sender c/lobby hello world").unwrap(),
            Some(ConsoleCommand::Push {
                source: "sender".to_string(),
                channel: "c/lobby".to_string(),
                text: "hello world".to_string()
            })
        );
        assert_eq!(
            parse_console_command("list c/lobby").unwrap(),
            Some(ConsoleCommand::Status {
                keyword: Some("c/lobby".to_string())
            })
        );
        assert_eq!(
            parse_console_command("stop deploy").unwrap(),
            Some(ConsoleCommand::Shutdown {
                reason: "deploy".to_string()
            })
        );
    }

    #[tokio::test]
    async fn service_routes_messages_and_excludes_sender() {
        let (sender_a, mut receiver_a) = mpsc::channel(OUTBOUND_QUEUE_SIZE);
        let (sender_b, mut receiver_b) = mpsc::channel(OUTBOUND_QUEUE_SIZE);
        let (sender_c, mut receiver_c) = mpsc::channel(OUTBOUND_QUEUE_SIZE);
        let mut state = ServiceState::new();

        state.handle_event(ServiceEvent::Connected {
            connection_id: 1,
            peer: None,
            sender: sender_a,
        });
        state.handle_event(ServiceEvent::Connected {
            connection_id: 2,
            peer: None,
            sender: sender_b,
        });
        state.handle_event(ServiceEvent::Connected {
            connection_id: 3,
            peer: None,
            sender: sender_c,
        });
        state.handle_event(ServiceEvent::Inbound {
            connection_id: 1,
            message: ClientMessage::Hello {
                ep_id: "a".to_string(),
            },
        });
        state.handle_event(ServiceEvent::Inbound {
            connection_id: 2,
            message: ClientMessage::Hello {
                ep_id: "b".to_string(),
            },
        });
        state.handle_event(ServiceEvent::Inbound {
            connection_id: 3,
            message: ClientMessage::Hello {
                ep_id: "c".to_string(),
            },
        });
        for connection_id in [1, 2, 3] {
            state.handle_event(ServiceEvent::Inbound {
                connection_id,
                message: ClientMessage::Subscribe {
                    channel: "room".to_string(),
                    valid_secs: 60,
                    valid_nanos: 0,
                },
            });
        }

        state.handle_event(ServiceEvent::Inbound {
            connection_id: 1,
            message: ClientMessage::Transmit {
                channel: "room".to_string(),
                text: "hello".to_string(),
            },
        });

        assert!(receiver_a.try_recv().is_err());
        assert_eq!(
            receiver_b.try_recv().unwrap(),
            OutboundMessage::Text("1r|1|a|4|room|5|hello".to_string())
        );
        assert_eq!(
            receiver_c.try_recv().unwrap(),
            OutboundMessage::Text("1r|1|a|4|room|5|hello".to_string())
        );
    }

    #[tokio::test]
    async fn service_cleans_expired_and_replaces_duplicate_endpoint() {
        let (sender_a1, mut receiver_a1) = mpsc::channel(OUTBOUND_QUEUE_SIZE);
        let (sender_a2, _receiver_a2) = mpsc::channel(OUTBOUND_QUEUE_SIZE);
        let mut state = ServiceState::new();

        state.handle_event(ServiceEvent::Connected {
            connection_id: 1,
            peer: None,
            sender: sender_a1,
        });
        state.handle_event(ServiceEvent::Inbound {
            connection_id: 1,
            message: ClientMessage::Hello {
                ep_id: "a".to_string(),
            },
        });
        state.handle_event(ServiceEvent::Inbound {
            connection_id: 1,
            message: ClientMessage::Subscribe {
                channel: "room".to_string(),
                valid_secs: 0,
                valid_nanos: 1,
            },
        });

        time::sleep(Duration::from_millis(2)).await;
        state.handle_event(ServiceEvent::CleanupExpired);
        assert_eq!(state.channel_count(), 0);

        state.handle_event(ServiceEvent::Connected {
            connection_id: 2,
            peer: None,
            sender: sender_a2,
        });
        state.handle_event(ServiceEvent::Inbound {
            connection_id: 2,
            message: ClientMessage::Hello {
                ep_id: "a".to_string(),
            },
        });
        assert_eq!(state.endpoint_count(), 1);
        assert_eq!(
            receiver_a1.try_recv().unwrap(),
            OutboundMessage::Close {
                kind: CloseKind::Error,
                reason: "Duplicate endpoint identity".to_string()
            }
        );
    }

    #[tokio::test]
    async fn websocket_integration_routes_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (event_tx, event_rx) = service_channel();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        tokio::spawn(run_service(event_rx, shutdown_tx.clone()));
        tokio::spawn(run_accept_loop(listener, event_tx.clone(), shutdown_rx));

        let (mut client_a, _) = connect_async(format!("ws://{}", addr)).await.unwrap();
        let (mut client_b, _) = connect_async(format!("ws://{}", addr)).await.unwrap();
        let (mut client_c, _) = connect_async(format!("ws://{}", addr)).await.unwrap();

        client_a
            .send(WsMessage::Text("1h|1|a".into()))
            .await
            .unwrap();
        client_b
            .send(WsMessage::Text("1h|1|b".into()))
            .await
            .unwrap();
        client_c
            .send(WsMessage::Text("1h|1|c".into()))
            .await
            .unwrap();
        client_a
            .send(WsMessage::Text("1c|4|room|60|0".into()))
            .await
            .unwrap();
        client_b
            .send(WsMessage::Text("1c|4|room|60|0".into()))
            .await
            .unwrap();
        client_c
            .send(WsMessage::Text("1c|5|other|60|0".into()))
            .await
            .unwrap();
        time::sleep(Duration::from_millis(100)).await;
        client_a
            .send(WsMessage::Text("1t|4|room|5|hello".into()))
            .await
            .unwrap();

        let received_b = timeout(Duration::from_secs(2), client_b.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(received_b, WsMessage::Text("1r|1|a|4|room|5|hello".into()));
        assert!(timeout(Duration::from_millis(100), client_a.next())
            .await
            .is_err());
        assert!(timeout(Duration::from_millis(100), client_c.next())
            .await
            .is_err());

        event_tx
            .send(ServiceEvent::Console(ConsoleCommand::Shutdown {
                reason: "test shutdown".to_string(),
            }))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn websocket_rejects_binary_frame() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (event_tx, event_rx) = service_channel();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        tokio::spawn(run_service(event_rx, shutdown_tx.clone()));
        tokio::spawn(run_accept_loop(listener, event_tx.clone(), shutdown_rx));

        let (mut client, _) = connect_async(format!("ws://{}", addr)).await.unwrap();
        client
            .send(WsMessage::Binary(vec![1, 2, 3].into()))
            .await
            .unwrap();
        let received = timeout(Duration::from_secs(2), client.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(received, WsMessage::Close(_)));

        event_tx
            .send(ServiceEvent::Console(ConsoleCommand::Shutdown {
                reason: "test shutdown".to_string(),
            }))
            .await
            .unwrap();
    }
}

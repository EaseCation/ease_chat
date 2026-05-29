use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ease_chat_nexus::OUTBOUND_QUEUE_SIZE;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, watch};
use tokio::time;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug, Clone)]
struct Config {
    url: String,
    clients: usize,
    publishers: usize,
    channels: usize,
    duration: Duration,
    message_size: usize,
    target_rate: Option<u64>,
    connect_parallel: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:6500".to_string(),
            clients: 350,
            publishers: 10,
            channels: 1,
            duration: Duration::from_secs(30),
            message_size: 64,
            target_rate: Some(1_000),
            connect_parallel: 128,
        }
    }
}

#[derive(Default)]
struct Counters {
    connected: AtomicU64,
    connect_failed: AtomicU64,
    sent: AtomicU64,
    received: AtomicU64,
    send_failed: AtomicU64,
    receive_failed: AtomicU64,
    closed_midrun: AtomicU64,
    stopping: AtomicBool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse()?;
    let expected_fanout = expected_average_fanout(&config);

    println!(
        "stress config: url={} clients={} publishers={} channels={} duration={}s message_size={} target_rate={} connect_parallel={} expected_fanout={:.2}",
        config.url,
        config.clients,
        config.publishers,
        config.channels,
        config.duration.as_secs(),
        config.message_size,
        config
            .target_rate
            .map(|rate| rate.to_string())
            .unwrap_or_else(|| "unlimited".to_string()),
        config.connect_parallel,
        expected_fanout,
    );

    let counters = Arc::new(Counters::default());
    let (stop_tx, stop_rx) = watch::channel(false);
    let started_at = Instant::now();
    let mut client_tasks = Vec::with_capacity(config.clients);
    let mut next_client = 0;

    while next_client < config.clients {
        let batch_end = (next_client + config.connect_parallel).min(config.clients);
        let mut batch = Vec::with_capacity(batch_end - next_client);
        for client_id in next_client..batch_end {
            let config = config.clone();
            let counters = counters.clone();
            let stop_rx = stop_rx.clone();
            batch.push(tokio::spawn(async move {
                run_client(client_id, config, counters, stop_rx).await;
            }));
        }
        for task in batch {
            client_tasks.push(task);
        }
        next_client = batch_end;
        time::sleep(Duration::from_millis(100)).await;
    }

    let mut last_sent = 0;
    let mut last_received = 0;
    let mut tick = time::interval(Duration::from_secs(1));
    loop {
        tick.tick().await;
        let elapsed = started_at.elapsed();
        let sent = counters.sent.load(Ordering::Relaxed);
        let received = counters.received.load(Ordering::Relaxed);
        let expected = (sent as f64 * expected_fanout).round() as u64;
        println!(
            "t={:>3}s connected={} sent/s={} recv/s={} sent_total={} recv_total={} expected_recv~{} send_fail={} recv_fail={} closed_midrun={}",
            elapsed.as_secs(),
            counters.connected.load(Ordering::Relaxed),
            sent.saturating_sub(last_sent),
            received.saturating_sub(last_received),
            sent,
            received,
            expected,
            counters.send_failed.load(Ordering::Relaxed),
            counters.receive_failed.load(Ordering::Relaxed),
            counters.closed_midrun.load(Ordering::Relaxed),
        );
        last_sent = sent;
        last_received = received;
        if elapsed >= config.duration {
            break;
        }
    }

    counters.stopping.store(true, Ordering::Relaxed);
    let _ = stop_tx.send(true);
    for task in client_tasks {
        let _ = time::timeout(Duration::from_secs(5), task).await;
    }

    let sent = counters.sent.load(Ordering::Relaxed);
    let received = counters.received.load(Ordering::Relaxed);
    let expected = (sent as f64 * expected_fanout).round() as u64;
    println!(
        "summary: elapsed={:.2}s connected={} connect_failed={} sent={} received={} expected_recv~{} delivery_ratio={:.4} send_failed={} receive_failed={} closed_midrun={}",
        started_at.elapsed().as_secs_f64(),
        counters.connected.load(Ordering::Relaxed),
        counters.connect_failed.load(Ordering::Relaxed),
        sent,
        received,
        expected,
        if expected == 0 {
            1.0
        } else {
            received as f64 / expected as f64
        },
        counters.send_failed.load(Ordering::Relaxed),
        counters.receive_failed.load(Ordering::Relaxed),
        counters.closed_midrun.load(Ordering::Relaxed),
    );
    Ok(())
}

async fn run_client(
    client_id: usize,
    config: Config,
    counters: Arc<Counters>,
    mut stop_rx: watch::Receiver<bool>,
) {
    let connect_result = connect_async(&config.url).await;
    let (socket, _) = match connect_result {
        Ok(result) => result,
        Err(error) => {
            counters.connect_failed.fetch_add(1, Ordering::Relaxed);
            eprintln!("connect failed for client {}: {}", client_id, error);
            return;
        }
    };

    counters.connected.fetch_add(1, Ordering::Relaxed);
    let (mut writer, mut reader) = socket.split();
    let (write_tx, mut write_rx) = mpsc::channel::<String>(OUTBOUND_QUEUE_SIZE);
    let name = format!("stress-{}", client_id);
    let channel = format!("stress/{}", client_id % config.channels);
    let is_publisher = client_id < config.publishers;

    let writer_counters = counters.clone();
    let writer_task = tokio::spawn(async move {
        while let Some(text) = write_rx.recv().await {
            if writer.send(Message::Text(text.into())).await.is_err() {
                writer_counters.send_failed.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
        let _ = writer.close().await;
    });

    let reader_counters = counters.clone();
    let reader_task = tokio::spawn(async move {
        while let Some(message) = reader.next().await {
            match message {
                Ok(Message::Text(_)) => {
                    reader_counters.received.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Message::Close(_)) => {
                    if !reader_counters.stopping.load(Ordering::Relaxed) {
                        reader_counters
                            .closed_midrun
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    break;
                }
                Ok(_) => {}
                Err(_) => {
                    reader_counters
                        .receive_failed
                        .fetch_add(1, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    send_text(&write_tx, hello_message(&name), &counters).await;
    send_text(&write_tx, subscribe_message(&channel, 600), &counters).await;

    if is_publisher {
        run_publisher(
            client_id,
            config,
            counters.clone(),
            write_tx.clone(),
            stop_rx.clone(),
        )
        .await;
    } else {
        let _ = stop_rx.changed().await;
    }

    drop(write_tx);
    let _ = writer_task.await;
    let _ = reader_task.await;
}

async fn run_publisher(
    client_id: usize,
    config: Config,
    counters: Arc<Counters>,
    write_tx: mpsc::Sender<String>,
    mut stop_rx: watch::Receiver<bool>,
) {
    let channel = format!("stress/{}", client_id % config.channels);
    let payload = build_payload(client_id, config.message_size);
    if let Some(target_rate) = config.target_rate {
        let per_publisher = (target_rate / config.publishers.max(1) as u64).max(1);
        let interval = Duration::from_nanos((1_000_000_000_u64 / per_publisher).max(1));
        let mut ticker = time::interval(interval);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    send_transmit(&write_tx, transmit_message(&channel, &payload), &counters).await;
                }
                changed = stop_rx.changed() => {
                    if changed.is_err() || *stop_rx.borrow() {
                        break;
                    }
                }
            }
        }
    } else {
        loop {
            tokio::select! {
                changed = stop_rx.changed() => {
                    if changed.is_err() || *stop_rx.borrow() {
                        break;
                    }
                }
                result = write_tx.send(transmit_message(&channel, &payload)) => {
                    if result.is_ok() {
                        counters.sent.fetch_add(1, Ordering::Relaxed);
                    } else {
                        counters.send_failed.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }
    }
}

async fn send_text(write_tx: &mpsc::Sender<String>, text: String, counters: &Counters) {
    if write_tx.send(text).await.is_err() {
        counters.send_failed.fetch_add(1, Ordering::Relaxed);
    }
}

async fn send_transmit(write_tx: &mpsc::Sender<String>, text: String, counters: &Counters) {
    if write_tx.send(text).await.is_ok() {
        counters.sent.fetch_add(1, Ordering::Relaxed);
    } else {
        counters.send_failed.fetch_add(1, Ordering::Relaxed);
    }
}

fn hello_message(name: &str) -> String {
    format!("1h|{}|{}", name.len(), name)
}

fn subscribe_message(channel: &str, seconds: u64) -> String {
    format!("1c|{}|{}|{}|0", channel.len(), channel, seconds)
}

fn transmit_message(channel: &str, payload: &str) -> String {
    format!(
        "1t|{}|{}|{}|{}",
        channel.len(),
        channel,
        payload.len(),
        payload
    )
}

fn build_payload(client_id: usize, message_size: usize) -> String {
    let prefix = format!("client={};", client_id);
    if prefix.len() >= message_size {
        return prefix;
    }
    let fill = "x".repeat(message_size - prefix.len());
    format!("{}{}", prefix, fill)
}

fn expected_average_fanout(config: &Config) -> f64 {
    if config.clients == 0 || config.publishers == 0 {
        return 0.0;
    }
    let mut total = 0_u64;
    for publisher_id in 0..config.publishers {
        let channel_index = publisher_id % config.channels;
        let subscribers = (0..config.clients)
            .filter(|client_id| client_id % config.channels == channel_index)
            .count();
        total += subscribers.saturating_sub(1) as u64;
    }
    total as f64 / config.publishers as f64
}

impl Config {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut config = Config::default();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--url" => config.url = take_value(&mut args, "--url")?,
                "--clients" => config.clients = take_value(&mut args, "--clients")?.parse()?,
                "--publishers" => {
                    config.publishers = take_value(&mut args, "--publishers")?.parse()?;
                }
                "--channels" => config.channels = take_value(&mut args, "--channels")?.parse()?,
                "--duration" => {
                    let seconds: u64 = take_value(&mut args, "--duration")?.parse()?;
                    config.duration = Duration::from_secs(seconds);
                }
                "--message-size" => {
                    config.message_size = take_value(&mut args, "--message-size")?.parse()?;
                }
                "--target-rate" => {
                    let value = take_value(&mut args, "--target-rate")?;
                    config.target_rate = if value == "max" || value == "0" {
                        None
                    } else {
                        Some(value.parse()?)
                    };
                }
                "--connect-parallel" => {
                    config.connect_parallel =
                        take_value(&mut args, "--connect-parallel")?.parse()?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(format!("Unknown argument: {}", other).into()),
            }
        }
        if config.clients == 0 {
            return Err("--clients must be greater than 0".into());
        }
        if config.publishers == 0 || config.publishers > config.clients {
            return Err("--publishers must be between 1 and clients".into());
        }
        if config.channels == 0 {
            return Err("--channels must be greater than 0".into());
        }
        Ok(config)
    }
}

fn take_value(
    args: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("Missing value for {}", name).into())
}

fn print_help() {
    println!(
        "EaseChat stress client\n\
         --url ws://127.0.0.1:6500\n\
         --clients 350\n\
         --publishers 10\n\
         --channels 1\n\
         --duration 30\n\
         --message-size 64\n\
         --target-rate 1000 | max\n\
         --connect-parallel 128"
    );
}

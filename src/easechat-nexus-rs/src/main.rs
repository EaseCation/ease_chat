use std::error::Error;
use std::io;
use std::thread;

use ease_chat_nexus::{
    parse_console_command, run_accept_loop, run_service, service_channel, ConsoleCommand,
    ServiceEvent, DEFAULT_ADDR,
};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_ADDR.to_string());
    let listener = TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;
    let (event_tx, event_rx) = service_channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    info!("[Module SOCKET] Listening on {}", local_addr);
    tokio::spawn(run_service(event_rx, shutdown_tx));

    let console_tx = event_tx.clone();
    thread::spawn(move || {
        info!("[Module CMDLINE] Started!");
        loop {
            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => match parse_console_command(&buffer) {
                    Ok(Some(command)) => {
                        let is_shutdown = matches!(command, ConsoleCommand::Shutdown { .. });
                        if console_tx
                            .blocking_send(ServiceEvent::Console(command))
                            .is_err()
                        {
                            break;
                        }
                        if is_shutdown {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => warn!("err: <Console Input> {}", error),
                },
                Err(error) => {
                    error!("Failed to read console input: {}", error);
                    break;
                }
            }
        }
    });

    run_accept_loop(listener, event_tx, shutdown_rx).await?;
    Ok(())
}

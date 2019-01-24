use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;
use std::thread;

struct MsgSignal {
    chan: String,
    msg: String,
}

struct ChanSignal {
    chan: String,
    valid_until: Instant,
}

struct MsgServiceFactory {
    log_tx: mpsc::Sender<LogSignal>,
}

struct MsgServiceHandler {
    log_tx: mpsc::Sender<LogSignal>,
    ws_sender: ws::Sender,
}

impl MsgServiceFactory {
    pub fn new(log_tx: mpsc::Sender<LogSignal>) -> Self {
        log_tx.send(LogSignal::ModuleStart(String::from("Message service"))).unwrap();
        Self { log_tx }
    }
}

impl ws::Factory for MsgServiceFactory {
    type Handler = MsgServiceHandler;
    fn connection_made(&mut self, ws_sender: ws::Sender) -> Self::Handler {
        Self::Handler { ws_sender, log_tx: self.log_tx.clone() }
    }
}

impl ws::Handler for MsgServiceHandler {

}

#[derive(Debug)]
enum LogSignal {
    ModuleStart(String),
    ShutdownRequest(),
}

fn main() {
    let (log_tx, log_rx) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(sig) = log_rx.recv() {
            println!("{:?}", sig);
        }
    });
    let addr = "0.0.0.0:6500";
    let log_tx1 = log_tx.clone();
    thread::spawn(move || {
        let fac = MsgServiceFactory::new(log_tx1);
        ws::WebSocket::new(fac).unwrap()
            .listen(addr).unwrap()
    });
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "q" => log_tx.send(LogSignal::ShutdownRequest()).unwrap(),
            _ => {}
        }
    }
}

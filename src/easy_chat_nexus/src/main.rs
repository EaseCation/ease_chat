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
    client_addr: Option<String>,
}

impl MsgServiceFactory {
    pub fn new(log_tx: mpsc::Sender<LogSignal>) -> Self {
        log_tx.send(LogSignal::ModuleStart(String::from("MSG-SERV"))).unwrap();
        Self { log_tx }
    }
}

impl ws::Factory for MsgServiceFactory {
    type Handler = MsgServiceHandler;
    fn connection_made(&mut self, ws_sender: ws::Sender) -> Self::Handler {
        Self::Handler { ws_sender, log_tx: self.log_tx.clone(), client_addr: None }
    }
}

impl ws::Handler for MsgServiceHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            self.log_tx.send(LogSignal::ConnectionOpen(addr.clone())).unwrap();
            self.client_addr = Some(addr);
        }
        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        self.log_tx.send(LogSignal::ConnectionClose(self.client_addr.clone().unwrap(), code, String::from(reason))).unwrap()
    }

    // fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
    //     unimplemented!()
    // }
}

#[derive(Debug)]
enum LogSignal {
    ModuleStart(String),
    ConnectionOpen(String),
    ConnectionClose(String, ws::CloseCode, String),
    ShutdownRequest(),
}

fn main() {
    let (log_tx, log_rx) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(sig) = log_rx.recv() {
            use LogSignal::*;
            match sig {
                ShutdownRequest() => {
                    println!("Shutting down...");
                    std::process::exit(0)
                },
                ModuleStart(meta) => println!("[Module {}] Started!", meta),
                ConnectionOpen(client_addr) => println!("[Client {}] Connection open!", client_addr),
                ConnectionClose(addr, code, reason) => println!("[Client {}] Connection closed, Code:[{:?}], Resion:[{}]", addr, code, reason),
                _ => {}
            }
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

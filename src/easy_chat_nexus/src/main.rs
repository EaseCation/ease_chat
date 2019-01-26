use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(Clone)]
struct Env {
    ep: Arc<Mutex<HashMap<String, ws::Sender>>>,
    chan: Arc<Mutex<HashMap<String, Vec<String>>>>, // chan_id -> ep_id
}

impl Env {
    pub fn new() -> Self {
        Env {
            ep: Arc::new(Mutex::new(HashMap::new())),
            chan: Arc::new(Mutex::new(HashMap::new()))
        }
    } 

    pub fn add_ep(&mut self, ep_id: String, sender: ws::Sender) {
        if let Ok(mut map) = self.ep.lock() {
            map.insert(ep_id, sender);
        }
    }

    // pub fn ep_reg_chan    
}

struct MsgServiceFactory {
    log_tx: mpsc::Sender<LogSignal>,
    msg_tx: mpsc::Sender<MsgSignal>,
    env: Env,
}

#[derive(Clone)]
struct MsgServiceHandler {
    log_tx: mpsc::Sender<LogSignal>,
    msg_tx: mpsc::Sender<MsgSignal>,
    ws_sender: ws::Sender,
    ep_id: Option<String>,
    env: Env,
}

impl MsgServiceFactory {
    pub fn new(log_tx: mpsc::Sender<LogSignal>, msg_tx: mpsc::Sender<MsgSignal>, env: Env) -> Self {
        log_tx.send(LogSignal::ModuleStart(String::from("MSG-SERV"))).unwrap();
        Self { log_tx, msg_tx, env }
    }
}

impl ws::Factory for MsgServiceFactory {
    type Handler = MsgServiceHandler;
    fn connection_made(&mut self, ws_sender: ws::Sender) -> Self::Handler {
        Self::Handler { 
            ws_sender, 
            log_tx: self.log_tx.clone(), 
            msg_tx: self.msg_tx.clone(), 
            ep_id: None,
            env: self.env.clone()
        }
    }
}

impl ws::Handler for MsgServiceHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            self.log_tx.send(LogSignal::ConnectionOpen(addr.clone())).unwrap();
            Ok(())
        } else {
            self.ws_sender.close(ws::CloseCode::Status)
        }
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        self.log_tx.send(LogSignal::ConnectionClose(self.ep_id.clone(), code, String::from(reason))).unwrap()
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let ws::Message::Text(text) = msg {
            self.handle_message_signal(text.chars().collect())
        } else {
            self.ws_sender.close_with_reason(ws::CloseCode::Unsupported, "Please input as string text")
        }
    }
}

impl MsgServiceHandler {
    // message string format: version+type|data
    // data of text: len|chan_id|len|msg_str
    // data of chan: len|chan_id|u64|u32(duration sec/nanos since unix epoch)
    // 1h|16|eafc5479a7e9f012
    // 1t|7|c/lobby|10|helloworld
    // 1c|7|c/lobby|1548507103|2140083600
    #[inline]
    fn handle_message_signal(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        match text.pop_front() {
            Some('1') => self.handle_v1(text),
            _ => self.ws_sender.close_with_reason(ws::CloseCode::Protocol, "Protocol other than '1' is not supported")
        }
    }

    #[inline]
    fn handle_v1(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        match (text.pop_front(), text.pop_front()) {
            (Some('h'), Some('|')) => self.handle_v1_handshake(text),
            (Some('t'), Some('|')) => self.handle_v1_text(text),
            (Some('c'), Some('|')) => self.handle_v1_chan(text),
            _ => self.ws_sender.close_with_reason(ws::CloseCode::Invalid, "Invalid message type: expected 't' or 'c'"),
        }
    }

    #[inline]
    fn read_number(text: &mut VecDeque<char>) -> u64 {
        let mut cur = text.pop_front();
        let mut ans = 0;
        while let Some(c) = cur {
            if let Some(digit) = c.to_digit(10) {
                ans *= 10;
                ans += digit as u64;
                cur = text.pop_front();
            } else {
                return ans;
            }
        };
        return ans;
    }

    #[inline]
    fn read_string(text: &mut VecDeque<char>) -> String {
        let cap = Self::read_number(text);
        let mut ans = String::with_capacity(cap as usize);
        for _i in 0..cap {
            if let Some(ch) = text.pop_front() {
                ans.push(ch)
            }
        }
        ans
    }

    // 16|eafc5479a7e9f012
    #[inline]
    fn handle_v1_handshake(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        let ep_id = Self::read_string(&mut text);
        self.ep_id = Some(ep_id.clone());
        self.env.add_ep(ep_id.clone(), self.ws_sender.clone());
        self.log_tx.send(LogSignal::ConnectionIdentified(ep_id)).unwrap();
        Ok(())
    }

    // 7|c/lobby|10|helloworld
    #[inline]
    fn handle_v1_text(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        if let Some(src_ep_id) = self.ep_id.clone() {
            let chan_id = Self::read_string(&mut text);
            let msg = Self::read_string(&mut text);
            self.msg_tx.send(MsgSignal::Text { src_ep_id, chan_id, msg }).unwrap();
            Ok(())
        } else {
            self.ws_sender.close_with_reason(ws::CloseCode::Status, "Connection unidentified")
        }
    }

    // 7|c/lobby|1548507103|2140083600
    #[inline]
    fn handle_v1_chan(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        if let Some(src_ep_id) = self.ep_id.clone() {
            let chan_id = Self::read_string(&mut text);
            let valid_until_sec = Self::read_number(&mut text);
            let valid_until_nanos = Self::read_number(&mut text);
            self.msg_tx.send(MsgSignal::Chan { src_ep_id, chan_id, valid_until_sec, valid_until_nanos }).unwrap();
            Ok(())
        } else {
            self.ws_sender.close_with_reason(ws::CloseCode::Status, "Connection unidentified")
        }
    }
}


#[derive(Debug)]
enum LogSignal {
    ModuleStart(String),
    ConnectionOpen(String),
    ConnectionIdentified(String),
    ConnectionClose(Option<String>, ws::CloseCode, String),
    ShutdownRequest(),
}

enum MsgSignal {
    Text {
        src_ep_id: String,
        chan_id: String,
        msg: String,
    },
    Chan {
        src_ep_id: String,
        chan_id: String,
        valid_until_sec: u64,
        valid_until_nanos: u64,
    }
}

fn main() {
    let (log_tx, log_rx) = mpsc::channel();
    let (msg_tx, msg_rx) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(sig) = log_rx.recv() {
            use LogSignal::*;
            match sig {
                ShutdownRequest() => {
                    println!("Shutting down...");
                    std::process::exit(0)
                },
                ModuleStart(meta) => 
                    println!("[Module {}] Started!", meta),
                ConnectionOpen(client_addr) => 
                    println!("[Client Addr {}] Connection open!", client_addr),
                ConnectionIdentified(ep_id) => 
                    println!("[Client EPID {}] Connection identified!", ep_id),
                ConnectionClose(addr, code, reason) => {
                    if let Some(ep_id) = addr {
                        println!("[Client EPID {}] Connection closed, Code:[{:?}], Reason:[{}]", ep_id, code, reason)
                    } else { 
                        println!("[Unidentified Client] Connection closed, Code:[{:?}], Reason:[{}]", code, reason)
                    }
                }
            }
        }
    });
    let addr = "0.0.0.0:6500";
    let log_tx1 = log_tx.clone();
    let msg_tx1 = msg_tx.clone();
    let env = Env::new();
    thread::spawn(move || {
        let fac = MsgServiceFactory::new(log_tx1, msg_tx1, env);
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

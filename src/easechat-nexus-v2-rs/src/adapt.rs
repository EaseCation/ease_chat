use super::service;
use std::collections::VecDeque;

pub(crate) struct WsFactoryAdapt<F> {
    inner: F,
    srv: service::Service,
}

impl<F> WsFactoryAdapt<F> {
    fn new(inner: F, srv: service::Service) -> Self {
        Self { inner, srv }
    }
} 

impl<F> ws::Factory for WsFactoryAdapt<F> 
where 
    F: super::Factory
{
    type Handler = WsHandlerAdapt<F::Handler>;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        let inner = self.inner.connection_made(super::Sender { inner: sender.clone() });
        WsHandlerAdapt { inner, ws_sender: sender, echat_id: None, srv: self.srv.clone() }
    }
    
    fn connection_lost(&mut self, handler: Self::Handler) {
        self.inner.connection_lost(handler.inner)
    }
}

pub struct WsHandlerAdapt<H> {
    inner: H,
    ws_sender: ws::Sender,
    srv: service::Service,
    echat_id: Option<String>
}

impl<H> ws::Handler for WsHandlerAdapt<H> 
where 
    H: super::Handler 
{
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        self.inner.on_open(super::Handshake { inner: shake });
        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        self.inner.on_close();
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        if let ws::Message::Text(text) = msg {
            self.handle_message_signal(text.chars().collect())
        } else {
            self.ws_sender.close_with_reason(ws::CloseCode::Unsupported, "Please send string message")
        }
    }
}

#[inline]
fn read_number(text: &mut VecDeque<char>) -> u64 {
    let mut cur = text.pop_front();
    while let Some(c) = cur {
        if c.to_digit(10).is_some() {
            break;
        } 
        cur = text.pop_front();
    }
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
    let cap = read_number(text);
    let mut ans = String::with_capacity(cap as usize);
    for _i in 0..cap {
        if let Some(ch) = text.pop_front() {
            ans.push(ch)
        }
    }
    ans
}

impl<H> WsHandlerAdapt<H> {
    #[inline]
    fn handle_message_signal(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        match text.pop_front() {
            Some('1') => self.handle_v1(text),
            _ => self.ws_sender.close_with_reason(ws::CloseCode::Protocol, "Protocol not supported")
        }
    }

    #[inline]
    fn handle_v1(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        match (text.pop_front(), text.pop_front()) {
            (Some('h'), Some('|')) => self.handle_v1_handshake(text),
            // (Some('c'), Some('|')) => self.handle_v1_listen_channel(text),
            // (Some('p'), Some('|')) => self.handle_v2_push_message(text),
            _ => self.ws_sender.close_with_reason(ws::CloseCode::Invalid, "Invalid message type"),
        }
    }

    #[inline]
    fn handle_v1_handshake(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
        let id = read_string(&mut text);
        self.echat_id = Some(id.clone());
        self.srv.register_client(id, super::Sender::new(self.ws_sender.clone()));
        Ok(())
    }

    // #[inline]
    // fn handle_v1_listen_channel(&mut self, mut text: VecDeque<char>) -> ws::Result<()> {
    //     if let Some(src_id) = self.echat_id.clone() {
    //         let chan_id = read_string(&mut text);
    //         let valid_until_sec = read_number(&mut text);
    //         let valid_until_nanos = read_number(&mut text);
    //         self.msg_tx.send(MsgSignal::Chan { src_id, chan_id, valid_until_sec, valid_until_nanos }).unwrap();
    //         Ok(())
    //     } else {
    //         self.ws_sender.close_with_reason(ws::CloseCode::Status, "Connection unidentified")
    //     }
    // }

}




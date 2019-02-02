mod adapt;
mod service;

pub struct EaseChat<F>
where
    F: Factory
{
    socket: ws::WebSocket<adapt::WsFactoryAdapt<F>>,
    srv: service::Service,
}

impl<F> EaseChat<F>
where
    F: Factory
{}


pub trait Factory {
    type Handler: Handler;

    fn connection_made(&mut self, _: Sender) -> Self::Handler;

    fn connection_lost(&mut self, _: Self::Handler) {}
}

pub trait Handler {
    fn on_open(&mut self, _: Handshake) {}

    fn on_close(&mut self) {}

    fn on_identify(&mut self, _id: String) {}

    fn on_listen_channel(&mut self, _target_id: String) {}

    fn on_push_message(&mut self, _dest_id: String, _msg: String) {}

    fn on_accept_message(&mut self, _src_id: String, _msg: String) {}
}

#[derive(Debug)]
pub struct Handshake {
    inner: ws::Handshake,
}

#[derive(Clone)]
pub struct Sender {
    inner: ws::Sender,
}

impl Sender {
    fn new(inner: ws::Sender) -> Self {
        Self { inner }
    }

    pub fn send_msg(&self, src_id: &str, dest_id: &str, msg: &str) -> ws::Result<()> {
        let string = format!("1r|{}|{}|{}|{}|{}|{}", src_id.len(), src_id, dest_id.len(), dest_id, msg.len(), msg); 
        self.inner.send(string)
    }  
}

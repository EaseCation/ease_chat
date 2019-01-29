mod adapt;
mod service;

pub struct EaseChat<F>
where
    F: Factory
{
    inner: ws::WebSocket<adapt::WsFactoryAdapt<F>>,
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

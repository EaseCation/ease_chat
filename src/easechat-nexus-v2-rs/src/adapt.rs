use super::service;

pub(crate) struct WsFactoryAdapt<F> {
    inner: F
}

impl<F> ws::Factory for WsFactoryAdapt<F> 
where 
    F: super::Factory
{
    type Handler = WsHandlerAdapt<F::Handler>;

    fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
        let inner = self.inner.connection_made(super::Sender { inner: sender });
        WsHandlerAdapt { inner }
    }
    
    fn connection_lost(&mut self, handler: Self::Handler) {
        self.inner.connection_lost(handler.inner)
    }
}

pub(crate) struct WsHandlerAdapt<H> {
    inner: H
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
        unimplemented!()
    }
}

use super::Sender;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub(crate) struct Service {
    id_to_sender: HashMap<String, Sender>,
    listen: HashMap<String, HashMap<Instant, (String, Sender)>>,
}

impl Service {
    pub fn identify(&mut self, id: String, sender: Sender) {
        
    }

    pub fn listen_channel(&mut self, src_id: String, target_id: String, valid_sec: u64, valid_nanos: u64) {
        
    }

    pub fn push_message(&mut self, src_id: String, dest_id: String, msg: String) {
        
    }

    pub fn accept_message(&mut self, src_id: String, msg: String) {
        
    }
}

use super::Sender;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Instant, Duration};

#[derive(Clone)]
pub(crate) struct Service {
    id_to_sender: Arc<RwLock<HashMap<String, Sender>>>,
    listen: Arc<RwLock<HashMap<String, HashMap<String, (Instant, Sender)>>>>,
}

impl Service {
    pub fn register_client(&mut self, id: String, sender: Sender) {
        self.id_to_sender.write().unwrap().insert(id, sender);
    }

    pub fn listen_channel(&mut self, src_id: String, target_id: String, valid_sec: u64, valid_nanos: u64) 
        -> Option<Instant>
    {
        self.id_to_sender.read().unwrap().get(&src_id).map(|sender| {
            let expire = Instant::now() + Duration::from_secs(valid_sec) + Duration::from_nanos(valid_nanos);
            self.listen.write().unwrap().entry(target_id).or_insert(HashMap::new())
                .insert(src_id, (expire, sender.clone()));
            expire
        })
    }

    pub fn push_message(&mut self, src_id: String, dest_id: String, msg: String) 
        -> ws::Result<usize>    
    {
        let map = self.listen.read().unwrap();
        let now = Instant::now();
        let mut cnt = 0;
        if let Some(senders) = map.get(&dest_id) {
            for (ep_id, (valid_until, sender)) in senders.iter() {
                if valid_until >= &now {
                    if ep_id != &src_id  {
                        sender.send_msg(&src_id, &dest_id, &msg)?;
                        cnt += 1
                    }
                } else {
                    if let Some(mp) = self.listen.write().unwrap().get_mut(&dest_id) {
                        mp.remove(ep_id);
                    }
                }
            }
        }
        Ok(cnt)
    }

    pub fn unregister_client(&mut self, src_id: String) {
        self.id_to_sender.write().unwrap().remove(&src_id);
        for (_chan, map) in self.listen.write().unwrap().iter_mut() {
            map.retain(|inner_ep_id, _instant_sender| &src_id != inner_ep_id);
        }
    }
}

pub enum Signal {
    Register {
        id: String,
        sender: Sender,
    },
    Listen {
        src_id: String,
        target_id: String,
        valid_sec: u64,
        valid_nanos: u64,
    },
    Push {
        src_id: String,
        dest_id: String,
        msg: String,
    },
    Accept {
        src_id: String,
        msg: String,
    },
    Unregister {
        src_id: String,
    }
}

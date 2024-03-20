use tokio::sync::mpsc;

use super::configstate::RuleSetID;
use std::collections::BTreeMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

struct LiveStateInner {
    pub clients: BTreeMap<RuleSetID, BTreeMap<IpAddr, mpsc::Sender<()>>>,
}

impl LiveStateInner {
    pub fn new() -> Self {
        LiveStateInner {
            clients: BTreeMap::new(),
        }
    }

    pub fn add_client(&mut self, ip: IpAddr, ruleset: RuleSetID) -> mpsc::Receiver<()> {
        let (tx, rx) = mpsc::channel(10);
        self.clients
            .entry(ruleset.clone())
            .or_insert_with(BTreeMap::new)
            .insert(ip, tx);

        rx
    }

    pub fn remove_client(&mut self, ip: IpAddr, ruleset: RuleSetID) {
        if let Some(clients) = self.clients.get_mut(&ruleset) {
            clients.remove(&ip);
            if clients.is_empty() {
                self.clients.remove(&ruleset);
            }
        }
    }
}

pub struct LiveState {
    l: Arc<Mutex<LiveStateInner>>,
}

impl LiveState {
    pub fn new() -> Self {
        LiveState {
            l: Arc::new(Mutex::new(LiveStateInner::new())),
        }
    }

    pub fn add_client(&self, ip: IpAddr, ruleset: RuleSetID) -> mpsc::Receiver<()> {
        self.l.lock().unwrap().add_client(ip, ruleset)
    }

    pub fn remove_client(&self, ip: IpAddr, ruleset: RuleSetID) {
        self.l.lock().unwrap().remove_client(ip, ruleset);
    }
}
use crate::ServerPeer;
use actix::Message;
use actix::prelude::Addr;
use lib::prelude::*;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct Pending {
    pub(crate) owner: Option<Addr<ServerPeer>>,
    pub(crate) owner_id: u32,
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
    pub(crate) got_lock: bool,
}

pub struct LockManager {
    ///Field just for leader. Keys: enterprise_id, Values: Transactions in pending queue
    pendings: HashMap<u32, VecDeque<Pending>>,
    ///Trasactions with lock granted. Keys: enterprise_id, Values: transaction_id
    current_transactions: HashMap<u32, String>,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            pendings: HashMap::new(),
            current_transactions: HashMap::new(),
        }
    }

    pub fn add_pending(&mut self, enterprise_id: u32, pending: Pending) {
        self.pendings
            .entry(enterprise_id)
            .or_insert_with(VecDeque::new)
            .push_back(pending);
    }

    pub fn get_pending_mut(&mut self, enterprise_id: &u32) -> Option<&mut VecDeque<Pending>> {
        self.pendings.get_mut(enterprise_id)
    }

    pub fn has_pending(&self, enterprise_id: &u32) -> bool {
        self.pendings.contains_key(enterprise_id)
    }

    pub fn pop_pending(&mut self, enterprise_id: &u32) -> Option<Pending> {
        if let Some(queue) = self.pendings.get_mut(enterprise_id) {
            queue.pop_front()
        } else {
            None
        }
    }

    pub fn peek_pending(&mut self, enterprise_id: &u32) -> Option<&mut Pending> {
        self.pendings
            .get_mut(enterprise_id)
            .and_then(|q| q.front_mut())
    }

    pub fn is_locked(&self, enterprise_id: &u32) -> bool {
        self.current_transactions.contains_key(enterprise_id)
    }

    pub fn grant_lock(&mut self, enterprise_id: u32, transaction_id: String) {
        log!(
            "[LM] Current transactions before granting lock: {:?}",
            self.current_transactions
        );
        self.current_transactions
            .insert(enterprise_id, transaction_id);
    }

    pub fn release_lock(&mut self, enterprise_id: &u32) -> Option<String> {
        log!(
            "[LM] Transactions before removing with id {}: {:?}",
            enterprise_id,
            self.current_transactions
        );
        self.current_transactions.remove(enterprise_id)
    }

    pub fn clear(&mut self) {
        self.current_transactions.clear();
        self.pendings.clear();
    }

    pub fn is_transaction_locked(&self, transaction_id: &str) -> bool {
        let enterprise_id = IdInterpreter::get_enterprise_id(transaction_id.to_string());
        let current = self.current_transactions.get(&enterprise_id);
        if let Some(current) = current {
            let verification = (current == transaction_id);
            if verification {
                log!("[LM] Verified the current transaction: {}", current);
            }
            verification
        } else {
            false
        }
    }

    pub fn get_locks_granted_to(&mut self, peer_id: u32) -> Vec<u32> {
        let granted = self
            .pendings
            .iter()
            .filter(|(_enterprise_id, pending)| {
                if let Some(p) = pending.front() {
                    return p.got_lock && p.owner_id == peer_id;
                }
                return false;
            })
            .map(|(enterprise_id, _)| *enterprise_id)
            .collect();
        self.pendings.iter_mut().for_each(|(_, pendings)| {
            pendings.retain(|p| p.owner_id != peer_id);
        });
        granted
    }
}

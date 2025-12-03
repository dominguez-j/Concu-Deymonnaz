use crate::locks::AcquireLock;
use crate::log;
use crate::managers::lock_manager::Pending;
use crate::managers::transaction_manager::{Transaction, TransactionManager};
use crate::transaction_with_id::{InternodeSelectWithId, SelectWithId, UpdateWithId};
use actix::prelude::*;
use lib::prelude::*;

impl Handler<Transaction> for TransactionManager {
    type Result = ();
    fn handle(&mut self, mut msg: Transaction, ctx: &mut Self::Context) -> Self::Result {
        log!("[TM {}] Handling transaction", self.id);
        match &msg.transaction {
            TransactionType::Create(create) => {
                log!(
                    "[TM {}] Handling enterprise creation with id {}",
                    self.id,
                    create.enterprise_id
                );
                self.repository_manager.do_send(create.clone());
                self.active_peers.iter().for_each(|(id, peer)| {
                    if *id != self.id {
                        peer.do_send(create.clone());
                    }
                })
            }
            TransactionType::Update(update) => {
                let base_id = update.get_update_id();
                let id = format!("{}+{}", base_id, self.update_counter);
                self.update_counter += 1;
                msg.initial_leader = self.leader;
                self.updates.insert(id.clone(), msg.clone());
                log!("[TM {}] Update transaction with id {}", self.id, id);
                ctx.address().do_send(UpdateWithId {
                    initial_leader: self.leader,
                    transaction_id: id,
                    update: update.clone(),
                });
            }
            TransactionType::Select(select) => {
                let id = format!("+{}", self.select_counter);
                self.select_counter += 1;
                self.selects.insert(id.clone(), msg.clone());
                self.repository_manager.do_send(SelectWithId {
                    transaction_id: id,
                    select: select.clone(),
                });
            }
            TransactionType::RequestData(request) => {
                let mut select = request.select.clone();
                let id = format!("{}*internode*{}", request.original_id, self.select_counter);
                log!("[TM {}] Requested data with id {}", self.id, id);
                select.update_transaction_id(id.clone());
                self.select_counter += 1;
                self.selects.insert(
                    id.clone(),
                    Transaction {
                        owner: msg.owner,
                        id: Some(id.clone()),
                        transaction: TransactionType::InternodeSelect(select.clone()),
                        initial_leader: request.initial_leader, // /ACÁ SE LLEVA CUENTA DEL LÍDER INICIAL CON EL QUE EL NODO ORIGEN HIZO LA REQUEST INTERNODO
                    },
                );
                self.repository_manager.do_send(InternodeSelectWithId {
                    transaction_id: id,
                    select: select.clone(),
                    initial_leader: request.initial_leader,
                });
            }
            _ => {}
        }
    }
}

impl Handler<Create> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: Create, _ctx: &mut Self::Context) -> Self::Result {
        self.repository_manager.do_send(msg.clone());
    }
}
///* Si hay un líder elegido se pide por internode el lock para la respectiva transacción, si el líder
/// es uno distinto que el current node, caso contrario, se enviará a sí mismo un "Pending" para que
///tal transacción sea encolada en el propio estado interno (que actuaría como estado interno del
///líder)
impl Handler<UpdateWithId> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: UpdateWithId, ctx: &mut Self::Context) -> Self::Result {
        if let Some(leader) = self.leader {
            log!("[TM {}] There is a leader", self.id);
            if let Some(peer) = self.active_peers.get(&leader) {
                log!("[TM {}] I am not the leader", self.id);
                log!(
                    "[TM {}] Requesting lock for transaction {}",
                    self.id,
                    msg.transaction_id
                );
                peer.do_send(AcquireLock {
                    from: self.id,
                    transaction_id: msg.transaction_id.clone(),
                    enterprise_id: IdInterpreter::get_enterprise_id(msg.transaction_id),
                })
            } else if self.id == leader {
                log!("[TM {}] I am the leader", self.id);
                log!(
                    "[TM {}] Requesting lock directly by sending a Pending to myself for transaction {}",
                    self.id,
                    msg.transaction_id
                );
                ctx.address().do_send(Pending {
                    owner: None,
                    owner_id: self.id,
                    transaction_id: msg.transaction_id.clone(),
                    enterprise_id: IdInterpreter::get_enterprise_id(msg.transaction_id),
                    got_lock: false,
                });
            }
        }
    }
}

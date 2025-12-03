use crate::TransactionManager;
use crate::locks::{LockGranted, NextLock, ReleaseLock};
use crate::log;
use crate::managers::lock_manager::Pending;
use crate::transaction_with_id::UpdateWithId;
use actix::prelude::*;
use lib::prelude::*;
use std::time::Duration;

impl Handler<Pending> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: Pending, ctx: &mut Self::Context) -> Self::Result {
        log!(
            "[TM {}] Received a Pending for transaction {} (now {})",
            self.id,
            msg.transaction_id,
            now_ms()
        );
        self.lock_manager
            .add_pending(msg.enterprise_id, msg.clone());
        if !self
            .lock_manager
            .peek_pending(&msg.enterprise_id)
            .unwrap()
            .got_lock
        {
            ctx.address().do_send(NextLock {
                enterprise_id: msg.enterprise_id,
            })
        } else {
            log!(
                "[TM {}] NextLock arrived when a transaction with id {} had the lock",
                self.id,
                msg.transaction_id
            );
        }
    }
}
///* Se otorga el lock a la siguiente transacción de una determinada empresa (en caso de haber
///transacciones pendientes), si el lock fue pedido por otro nodo, se le enviará un LockGranted
///a dicho nodo identificando la transacción a la que le fue otorgado dicho lock; si quien pidió el
///lock fue el current node, será avisado igualmente con un LockGranted por mensaje de actor para
///que continúe con la respectiva operación
impl Handler<NextLock> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: NextLock, ctx: &mut Self::Context) -> Self::Result {
        log!("[TM {}] Evaluating who to grant next lock", self.id);
        let Some(front) = self.lock_manager.peek_pending(&msg.enterprise_id) else {
            log!(
                "[TM {}] Queue for enterprise_id:{} is empty, not transactions to grant lock",
                self.id,
                msg.enterprise_id
            );
            return;
        };
        if !front.got_lock {
            front.got_lock = true;
            if let Some(owner) = &front.owner {
                log!(
                    "[TM {}] Sending LockGranted to peer for transaction {} (now {})",
                    self.id,
                    front.transaction_id,
                    now_ms()
                );
                owner.do_send(LockGranted {
                    enterprise_id: front.enterprise_id,
                    transaction_id: front.transaction_id.clone(),
                });
            } else {
                log!(
                    "[TM {}] Sending LockGranted to myself for transaction {} (now {})",
                    self.id,
                    front.transaction_id,
                    now_ms()
                );
                ctx.address().do_send(LockGranted {
                    enterprise_id: front.enterprise_id,
                    transaction_id: front.transaction_id.clone(),
                });
            }
        }
    }
}

impl Handler<LockGranted> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: LockGranted, _: &mut Self::Context) -> Self::Result {
        log!(
            "[TM {}] Received LockGranted from leader for transaction {} (now: {})",
            self.id,
            msg.transaction_id,
            now_ms(),
        );
        let tr = self
            .updates
            .get(&msg.transaction_id)
            .expect("Invalid transaction ID when lock granted");
        let TransactionType::Update(update) = tr.transaction.clone() else {
            panic!("Transaction is not an update when lock granted"); // Cannot happen
        };

        let enterprise_id = update.get_enterprise_id();
        if self.lock_manager.is_locked(&enterprise_id) {
            log!(
                "[TM {}] Warning: Transaction in current_transactions did not remove in previous operation",
                self.id
            );
            return;
        }
        self.lock_manager
            .grant_lock(enterprise_id, msg.transaction_id.to_string());

        self.repository_manager.do_send(UpdateWithId {
            initial_leader: tr.initial_leader,
            transaction_id: msg.transaction_id,
            update,
        });
    }
}

impl Handler<ReleaseLock> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: ReleaseLock, ctx: &mut Self::Context) {
        log!(
            "[TM {}] Received a lock release for transaction {} (now {})",
            self.id,
            msg.transaction_id,
            now_ms()
        );
        let enterprise_id = msg.enterprise_id;
        if let Some(pendings) = self.lock_manager.get_pending_mut(&enterprise_id) {
            if let Some(to_remove) = pendings.front() {
                log!(
                    "[TM {}] Trying to remove current transaction.\
                Expected id {} and got id {}",
                    enterprise_id,
                    to_remove.transaction_id,
                    msg.transaction_id
                );
                if msg.transaction_id == to_remove.transaction_id {
                    pendings.pop_front();
                    trace!(
                        "[TM {}] Transaction: {} removed from leader pending queue",
                        self.id, msg.transaction_id
                    );
                    ctx.address().do_send(NextLock { enterprise_id });
                } else {
                    log!(
                        "[TM {}] Warning: Trying to remove an invalid transaction {}",
                        self.id,
                        msg.transaction_id
                    );
                    log!(
                        "[TM {}] Transaction in pendings queue front: {}",
                        self.id,
                        to_remove.transaction_id
                    );
                }
            }
        }
    }
}

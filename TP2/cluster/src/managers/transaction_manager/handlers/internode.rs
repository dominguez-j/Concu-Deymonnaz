use crate::TransactionManager;
use crate::locks::ReleaseLock;
use crate::log;
use crate::messages::*;
use crate::server_peer::TransactionResponseFromInternode;
use actix::prelude::*;
use lib::messages::set_own_data::{SendBroadcastOfSetOwnData, SetOwnData};
use lib::prelude::*;
use lib::trace;

/// INTERNODE SEND BROADCAST SET OWN DATA
impl Handler<SendBroadcastOfSetOwnData> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: SendBroadcastOfSetOwnData, ctx: &mut Self::Context) -> Self::Result {
        let enterprise_id = msg.command.enterprise_id.clone();
        let tr_id_removed = self.lock_manager.release_lock(&enterprise_id);
        log!(
            "[TM {}] SendBroadcastOfSetOwnData received for transaction with id {:?}",
            self.id,
            tr_id_removed
        );
        if msg.do_broadcast && tr_id_removed.is_some() {
            log!("[TM {}] Starting broadcast of set own data", self.id);
            self.active_peers.iter().for_each(|(id, peer)| {
                if *id != self.id {
                    peer.do_send(msg.clone())
                }
            });
            self.repository_manager.do_send(msg.command.clone());
        } else if tr_id_removed.is_none() {
            log!(
                "[TM {}] Leader changed in the middle of transaction for enterprise with id {}",
                self.id,
                msg.command.enterprise_id
            );
        } else if !msg.do_broadcast {
            log!(
                "[TM {}] Broadcast disabled for transaction for enterprise with id {}",
                self.id,
                msg.command.enterprise_id
            );
        }
        log!(
            "[TM {}] Transaction ID removed: {:?}",
            self.id,
            tr_id_removed
        );
        if let Some(id_removed) = tr_id_removed {
            let removed = self
                .updates
                .remove(&id_removed)
                .expect("Missing update when broadcasting");
            let TransactionType::Update(update) = removed.transaction else {
                panic!(
                    "[TM {}] Invalid transaction type when broadcasting",
                    self.id
                );
            };
            if let Update::Payment { .. } = update {
                removed
                    .owner
                    .do_send(TransactionResponse::TransactionResultResponse {
                        transaction_id: id_removed.clone(),
                        // TODO: Remover esta copia innecesaria
                        result: TransactionResult::new(id_removed.clone(), msg.result),
                    })
            }
            trace!(
                "[TM {}] transaction_id: {}, removed from updates HashMap",
                self.id, id_removed
            );

            if let Some(leader) = self.leader {
                log!(
                    "[TM {}] Releasing lock to leader {} from transaction {} (now {})",
                    self.id,
                    leader,
                    id_removed,
                    now_ms()
                );
                if let Some(addr) = self.active_peers.get(&leader) {
                    addr.do_send(ReleaseLock {
                        enterprise_id,
                        transaction_id: id_removed,
                    });
                } else if self.id == leader {
                    ctx.address().do_send(ReleaseLock {
                        enterprise_id,
                        transaction_id: id_removed,
                    });
                }
            }
        }
    }
}
/// INTERNODE SET OWN DATA RECEIVED
impl Handler<SetOwnData> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: SetOwnData, _ctx: &mut Self::Context) -> Self::Result {
        self.repository_manager.do_send(msg.clone());
    }
}
/// INTERNODE SELECTS
///* Desde acá se realiza el envío de internode requests de data que no la tiene el Repository del
///current node (Selects o Updates) en forma de una "RequestData"
impl Handler<InternodeSelect> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: InternodeSelect, _: &mut Self::Context) -> Self::Result {
        self.active_peers.iter().for_each(|(_, peer)| {
            peer.do_send(RequestData {
                from: self.id,
                original_id: msg.get_transaction_id(),
                select: msg.clone(),
                initial_leader: msg.get_initial_leader(),
            });
        });
    }
}
/// RESPUESTAS
impl Handler<CardViewResponse> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: CardViewResponse, _: &mut Self::Context) -> Self::Result {
        let transaction_id = msg.transaction_id.clone();
        let res = msg.to_tr();
        if let Some(transaction) = self.selects.get(&transaction_id) {
            transaction.owner.do_send(res);
        }
        self.selects.remove(&transaction_id);
    }
}

impl Handler<EnterpriseViewResponse> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: EnterpriseViewResponse, _ctx: &mut Self::Context) -> Self::Result {
        let transaction_id = msg.transaction_id.clone();
        let res = msg.to_tr();
        if let Some(transaction) = self.selects.get(&transaction_id) {
            transaction.owner.do_send(res);
        }
        self.selects.remove(&transaction_id);
    }
}

impl Handler<InternodeResponse> for TransactionManager {
    type Result = ();
    fn handle(&mut self, msg: InternodeResponse, _: &mut Self::Context) -> Self::Result {
        log!("[TM {}] Message at InternodeResponse: {:?}", self.id, msg);
        let transaction = self
            .selects
            .get(&msg.get_transaction_id())
            .expect("Transaction not found when InternodeResponse");
        transaction.owner.do_send(msg);
    }
}

///* Todas los datos manejados acá son provenientes de requests previas (tanto para Selects originales
///como Updates), que el current node hizo en internode, debido a que él mismo no tenía la información
///en su RepositoryManager
impl Handler<TransactionResponseFromInternode> for TransactionManager {
    type Result = ();
    fn handle(
        &mut self,
        msg: TransactionResponseFromInternode,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let transaction_id = msg.original_id.clone();
        log!(
            "[TM {}] Received a transaction response from internode with transaction ID: {}",
            self.id,
            transaction_id
        );

        if !msg.response.check_if_data_is_valid() {
            log!(
                "[TM {}] Internode response was empty data for transaction with id {} - Dismissing",
                self.id,
                msg.original_id
            );
            return;
        }

        match msg.response {
            InternodeResponse::CardViewResponse { .. }
            | InternodeResponse::EnterpriseViewResponse { .. } => {
                if let Some(transaction) = self.selects.get(&transaction_id) {
                    if let Some(tr) = msg.response.to_tr() {
                        transaction.owner.do_send(tr);
                    }
                    self.selects.remove(&transaction_id);
                } else {
                    trace!(
                        "[TM {}] Ignoring internode response for transaction with id {}",
                        self.id, transaction_id
                    );
                }
            }
            InternodeResponse::CardViewForUpdateResponse {
                response,
                initial_leader,
                ..
            } => {
                if !self.updates.get(&transaction_id).is_some() {
                    trace!(
                        "[TM {}] Ignoring internode response for transaction with id {}",
                        self.id, transaction_id
                    );
                    return;
                }
                self.handle_card_view_for_update(
                    ctx.address().clone(),
                    response,
                    transaction_id,
                    initial_leader,
                );
            }
            InternodeResponse::EnterpriseViewForUpdateResponse {
                response,
                initial_leader,
                ..
            } => {
                if !self.updates.get(&transaction_id).is_some() {
                    trace!(
                        "[TM {}] Ignoring internode response for transaction with id {}",
                        self.id, transaction_id
                    );
                    return;
                }
                self.handle_enterprise_view_for_update(
                    ctx.address().clone(),
                    response,
                    transaction_id,
                    initial_leader,
                );
            }
            InternodeResponse::PaymentViewResponse {
                card_id,
                enterprise_id,
                response,
                initial_leader,
                ..
            } => {
                if !self.updates.get(&transaction_id).is_some() {
                    trace!(
                        "[TM {}] Ignoring internode response for transaction with id {}",
                        self.id, transaction_id
                    );
                    return;
                }
                self.handle_payment_view(
                    ctx.address().clone(),
                    response,
                    transaction_id,
                    card_id,
                    enterprise_id,
                    initial_leader,
                );
            }
        }
    }
}

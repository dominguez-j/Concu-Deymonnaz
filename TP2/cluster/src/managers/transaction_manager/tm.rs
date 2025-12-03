use crate::ConnectionManager;
use crate::RepositoryManager;
use crate::ServerPeer;
use crate::log;
use crate::managers::lock_manager::LockManager;
use actix::prelude::*;
use lib::messages::set_own_data::{SendBroadcastOfSetOwnData, SetOwnData, UpdateRegister};
use lib::prelude::*;
use lib::trace;
use std::collections::HashMap;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterPeerTM {
    pub id: u32,
    pub addr: Addr<ServerPeer>,
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct Transaction {
    pub(crate) owner: Addr<ServerPeer>,
    pub(crate) transaction: TransactionType,
    pub(crate) id: Option<String>,
    pub(crate) initial_leader: Option<u32>,
}

pub struct TransactionManager {
    pub(crate) id: u32,
    pub(crate) leader: Option<u32>,
    pub(crate) repository_manager: Addr<RepositoryManager>,
    ///All transaction Selects received from clients
    pub(crate) selects: HashMap<String, Transaction>,
    ///All transaction Updates received from clients
    pub(crate) updates: HashMap<String, Transaction>,
    pub(crate) select_counter: u32,
    pub(crate) update_counter: u32,
    pub(crate) lock_manager: LockManager,
    pub(crate) conn_manager: Addr<ConnectionManager>,
    pub(crate) active_peers: HashMap<u32, Addr<ServerPeer>>,
}

impl TransactionManager {
    pub fn new(
        id: u32,
        repository_manager: Addr<RepositoryManager>,
        conn_manager: Addr<ConnectionManager>,
    ) -> Addr<Self> {
        Self {
            id,
            leader: None,
            repository_manager,
            selects: HashMap::new(),
            updates: HashMap::new(),
            select_counter: 0,
            update_counter: 0,
            lock_manager: LockManager::new(),
            conn_manager,
            active_peers: HashMap::new(),
        }
        .start()
    }

    pub(crate) fn handle_card_view_for_update(
        &mut self,
        tm_addr: Addr<TransactionManager>,
        response: Option<CardViewResponseData>,
        transaction_id: String,
        initial_leader: Option<u32>,
    ) {
        let Some(response) = response else {
            log!(
                "[TM {}] Internode response for {} is none",
                self.id,
                transaction_id
            );
            return;
        };
        let Some(transaction) = self.updates.get(&transaction_id).cloned() else {
            log!(
                "[TM {}] Transaction not found for internode response: {}",
                self.id,
                transaction_id
            );
            return;
        };

        let TransactionType::Update(update) = transaction.transaction else {
            log!(
                "[TM {}] Invalid transaction type for CardViewForUpdateResponse: {}",
                self.id,
                transaction_id
            );
            return;
        };

        if let Update::CardLimitUpdate {
            enterprise_id,
            card_id,
            limit,
            update_type,
        } = update
        {
            let current_limit = response.card_limit.unwrap_or(0);
            let new_card_limit = match update_type {
                UpdateType::Set => limit,
                UpdateType::Increment => current_limit + limit,
                UpdateType::Decrement => current_limit.saturating_sub(limit),
            };
            let set_own_data = SetOwnData {
                enterprise_id,
                data: UpdateRegister::CardLimitOrUsage {
                    card_id,
                    card_limit: new_card_limit,
                    card_usage: response.card_usage,
                    enterprise_limit: response.enterprise_limit.unwrap_or(0),
                    enterprise_usage: response.enterprise_usage,
                },
            };
            // self.repository_manager.do_send(set_own_data.clone());   // /NO HAY QUE HACER EL LLAMADO A set_own_data DESDE ACÁ SINO DESDE EL BROADCAST DEL TM
            tm_addr.do_send(SendBroadcastOfSetOwnData {
                initial_leader,
                command: set_own_data,
                result: true,
                do_broadcast: true,
            })
        } else {
            log!(
                "[TM {}] Invalid update type for CardViewForUpdateResponse: {}",
                self.id,
                transaction_id
            );
        }
    }

    pub(crate) fn handle_enterprise_view_for_update(
        &mut self,
        tm_addr: Addr<TransactionManager>,
        response: Option<EnterpriseViewResponseData>,
        transaction_id: String,
        initial_leader: Option<u32>,
    ) {
        let res = response.expect("EnterpriseViewResponseData missing");
        let tr = self
            .updates
            .get(&transaction_id)
            .cloned()
            .expect("Transaction missing");
        let TransactionType::Update(update) = tr.transaction else {
            log!(
                "[TM {}] Invalid transaction type for EnterpriseViewForUpdateResponse: {}",
                self.id,
                transaction_id
            );
            return;
        };
        if let Update::EnterpriseLimitUpdate {
            limit,
            update_type,
            enterprise_id,
        } = update
        {
            let current_limit = res.limit;
            let new_limit = match update_type {
                UpdateType::Set => limit,
                UpdateType::Increment => current_limit + limit,
                UpdateType::Decrement => current_limit.saturating_sub(limit),
            };

            let set_own_data = SetOwnData {
                enterprise_id,
                data: UpdateRegister::EnterpriseLimit {
                    limit: new_limit,
                    usage: res.usage,
                },
            };
            // self.repository_manager.do_send(set_own_data.clone());   // /NO HAY QUE HACER EL LLAMADO A set_own_data DESDE ACÁ SINO DESDE EL BROADCAST DEL TM
            tm_addr.do_send(SendBroadcastOfSetOwnData {
                initial_leader,
                command: set_own_data,
                result: true,
                do_broadcast: true,
            })
        } else {
            log!(
                "[TM {}] Invalid update type for EnterpriseViewForUpdateResponse: {}",
                self.id,
                transaction_id
            );
        }
    }

    pub(crate) fn handle_payment_view(
        &mut self,
        tm_addr: Addr<TransactionManager>,
        response: Option<PaymentViewResponseInfo>,
        transaction_id: String,
        enterprise_id: u32,
        card_id: u32,
        initial_leader: Option<u32>,
    ) {
        let res = response.expect("PaymentView missing");
        log!(
            "[TM {}] Transaction id: {} - Updates keys: {:?}",
            transaction_id,
            self.id,
            self.updates.keys().collect::<Vec<_>>()
        );
        let tr = self
            .updates
            .get(&transaction_id)
            .cloned()
            .expect("Transaction missing");
        let TransactionType::Update(update) = tr.transaction else {
            trace!(
                "[TM {}] Invalid transaction type for PaymentView: {}",
                self.id, transaction_id
            );
            return;
        };
        if let Update::Payment {
            payment_id,
            card_id,
            enterprise_id,
            cost,
            transaction_type,
        } = update
        {
            let set_own_data = SetOwnData {
                enterprise_id,
                data: UpdateRegister::CardLimitOrUsage {
                    card_id,
                    card_limit: res.card_limit.unwrap_or(0),
                    card_usage: res.card_usage + cost,
                    enterprise_limit: res.enterprise_limit.unwrap_or(0),
                    enterprise_usage: res.enterprise_usage + cost,
                },
            };
            match transaction_type {
                PaymentType::ForcePayment => {
                    // self.repository_manager.do_send(set_own_data.clone());// /NO HAY QUE HACER EL LLAMADO A set_own_data DESDE ACÁ SINO DESDE EL BROADCAST DEL TM
                    tm_addr.do_send(SendBroadcastOfSetOwnData {
                        // /ASÍ SE HACE GENERAL PARA LOS CASOS EN QUE SE TIENE LA DATA O NO SE TIENE!
                        initial_leader,
                        command: set_own_data,
                        result: true,
                        do_broadcast: true,
                    })
                }
                PaymentType::PaymentVerification => {
                    if res.card_usage + cost > res.card_limit.unwrap_or(res.card_usage + cost)
                        || res.enterprise_usage + cost
                            > res.enterprise_limit.unwrap_or(res.enterprise_usage + cost)
                    {
                        log!("[TM {}] PaymentVerification failed, aborting", self.id);
                        tm_addr.do_send(SendBroadcastOfSetOwnData {
                            initial_leader,
                            command: set_own_data,
                            result: false,
                            do_broadcast: false,
                        });
                    } else {
                        log!(
                            "[TM {}] PaymentVerification succeeded, entering broadcast",
                            self.id
                        );
                        // self.repository_manager.do_send(set_own_data.clone());   // /NO HAY QUE HACER EL LLAMADO A set_own_data DESDE ACÁ SINO DESDE EL BROADCAST DEL TM
                        tm_addr.do_send(SendBroadcastOfSetOwnData {
                            initial_leader,
                            command: set_own_data,
                            result: true,
                            do_broadcast: true,
                        });
                    }
                }
            }
        };
    }
}

impl Actor for TransactionManager {
    type Context = Context<Self>;
}

use crate::RepositoryManager;
use crate::messages::*;
use crate::spending_info::SpendingInfoRegister;
use crate::transaction_with_id::*;
use actix::prelude::*;
use lib::messages::set_own_data::{SendBroadcastOfSetOwnData, SetOwnData, UpdateRegister};
use lib::prelude::*;
use std::collections::HashMap;

pub struct Repository {
    id: u32,
    current_usage: u32,
    limit: Option<u32>,
    cards: HashMap<u32, SpendingInfoRegister>,
    repository_manager: Addr<RepositoryManager>,
}

impl Repository {
    pub fn new(id: u32, repository_manager: Addr<RepositoryManager>) -> Addr<Self> {
        Self {
            id,
            current_usage: 0,
            limit: None,
            cards: HashMap::new(),
            repository_manager,
        }
        .start()
    }
    fn send_broadcast_to_set_own_data(
        manager: &mut Addr<RepositoryManager>,
        initial_leader: Option<u32>,
        reg: &mut SpendingInfoRegister,
        enterprise_id: u32,
        enterprise_limit: u32,
        enterprise_usage: u32,
        result: bool,
        do_broadcast: bool,
    ) {
        let set_own_data = SetOwnData {
            enterprise_id,
            data: UpdateRegister::CardLimitOrUsage {
                card_id: reg.get_card_id(),
                card_limit: reg.get_limit().unwrap_or(0),
                card_usage: reg.get_current_usage(),
                enterprise_limit,
                enterprise_usage,
            },
        };

        manager.do_send(SendBroadcastOfSetOwnData {
            command: set_own_data,
            initial_leader,
            result,
            do_broadcast,
        });
    }
}

impl Actor for Repository {
    type Context = Context<Self>;
}

impl Handler<Create> for Repository {
    type Result = ();
    fn handle(&mut self, msg: Create, _ctx: &mut Self::Context) -> Self::Result {
        self.limit = Some(msg.enterprise_limit);
        msg.card_limits.iter().for_each(|(id, limit)| {
            self.cards
                .insert(*id, SpendingInfoRegister::new(*id, Some(*limit)));
        });
        log!(
            "[R {}] Created enterprise with limit {:?} and cards: {:?}",
            msg.enterprise_id,
            self.limit,
            self.cards
        );
    }
}

impl Handler<SetOwnData> for Repository {
    type Result = ();
    fn handle(&mut self, msg: SetOwnData, _ctx: &mut Self::Context) -> Self::Result {
        match msg.data {
            UpdateRegister::EnterpriseLimit { limit, usage } => {
                self.limit = Some(limit);
                self.current_usage = usage;
            }
            UpdateRegister::CardLimitOrUsage {
                card_id,
                card_limit,
                card_usage,
                enterprise_limit,
                enterprise_usage,
            } => {
                if let Some(card) = self.cards.get_mut(&card_id) {
                    card.set_limit(Some(card_limit));
                    card.set_usage(card_usage);
                } else {
                    // /ESTE CASO NO DEBERÍA DE SUCEDER, SI NO HAY TARJETA, NO HAY Repository
                    let mut card = SpendingInfoRegister::new(card_id, Some(card_limit));
                    card.set_usage(self.current_usage);
                    self.cards.insert(card_id, card);
                    log!(
                        "[R {}] Warning: Card {} did not exist, but now is registered",
                        self.id,
                        card_id
                    );
                }
                self.limit = Some(enterprise_limit);
                self.current_usage = enterprise_usage;
            }
        }
    }
}

impl Handler<CardView> for Repository {
    type Result = ();
    fn handle(&mut self, msg: CardView, _: &mut Context<Self>) {
        let reg = self.cards.get(&msg.card_id).cloned();
        let data;
        if let Some(reg) = reg {
            data = Some(CardViewResponseData {
                enterprise_usage: self.current_usage,
                enterprise_limit: self.limit,
                card_usage: reg.get_current_usage(),
                card_limit: reg.get_limit(),
            })
        } else {
            data = None;
        }
        self.repository_manager.do_send(CardViewResponse {
            transaction_id: msg.transaction_id,
            enterprise_id: self.id,
            card_id: msg.card_id,
            response: data,
        });
    }
}

impl Handler<EnterpriseView> for Repository {
    type Result = ();
    fn handle(&mut self, msg: EnterpriseView, _: &mut Context<Self>) {
        self.repository_manager.do_send(EnterpriseViewResponse {
            transaction_id: msg.transaction_id,
            enterprise_id: self.id,
            data: Some(EnterpriseViewResponseData {
                usage: self.current_usage,
                limit: self.limit.unwrap_or(0),
            }),
        });
    }
}

impl Handler<InternodeSelectWithId> for Repository {
    type Result = ();
    fn handle(&mut self, msg: InternodeSelectWithId, _: &mut Self::Context) -> Self::Result {
        match msg.select {
            InternodeSelect::CardViewFromInternode {
                transaction_id,
                card_id,
                enterprise_id,
            } => {
                if let Some(res) = self.cards.get(&card_id) {
                    self.repository_manager
                        .do_send(InternodeResponse::CardViewResponse {
                            response: Some(CardViewResponseData {
                                card_usage: res.get_current_usage(),
                                card_limit: res.get_limit(),
                                enterprise_usage: self.current_usage,
                                enterprise_limit: self.limit,
                            }),
                            enterprise_id,
                            card_id,
                            transaction_id,
                        })
                } else {
                    self.repository_manager
                        .do_send(InternodeResponse::CardViewResponse {
                            response: None,
                            transaction_id,
                            enterprise_id,
                            card_id,
                        })
                }
            }
            InternodeSelect::CardViewForUpdateFromInternode {
                transaction_id,
                card_id,
                initial_leader,
                ..
            } => {
                if let Some(res) = self.cards.get(&card_id) {
                    self.repository_manager
                        .do_send(InternodeResponse::CardViewForUpdateResponse {
                            response: Some(CardViewResponseData {
                                card_usage: res.get_current_usage(),
                                card_limit: res.get_limit(),
                                enterprise_usage: self.current_usage,
                                enterprise_limit: self.limit,
                            }),
                            transaction_id,
                            initial_leader,
                        })
                } else {
                    self.repository_manager
                        .do_send(InternodeResponse::CardViewForUpdateResponse {
                            response: None,
                            transaction_id,
                            initial_leader,
                        })
                }
            }
            InternodeSelect::EnterpriseViewForUpdateFromInternode {
                transaction_id,
                initial_leader,
                ..
            } => self.repository_manager.do_send(
                InternodeResponse::EnterpriseViewForUpdateResponse {
                    response: Some(EnterpriseViewResponseData {
                        usage: self.current_usage,
                        limit: self.limit.unwrap_or(0),
                    }),
                    transaction_id,
                    initial_leader,
                },
            ),
            InternodeSelect::EnterpriseViewFromInternode { transaction_id, .. } => self
                .repository_manager
                .do_send(InternodeResponse::EnterpriseViewResponse {
                    response: Some(EnterpriseViewResponseData {
                        usage: self.current_usage,
                        limit: self.limit.unwrap_or(0),
                    }),
                    transaction_id,
                }),
            InternodeSelect::PaymentView {
                enterprise_id,
                card_id,
                transaction_id,
                initial_leader,
            } => {
                if let Some(res) = self.cards.get(&card_id) {
                    self.repository_manager
                        .do_send(InternodeResponse::PaymentViewResponse {
                            response: Some(PaymentViewResponseInfo {
                                card_limit: res.get_limit(),
                                card_usage: res.get_current_usage(),
                                enterprise_limit: self.limit,
                                enterprise_usage: self.current_usage,
                            }),
                            transaction_id,
                            enterprise_id,
                            card_id,
                            initial_leader,
                        })
                } else {
                    self.repository_manager
                        .do_send(InternodeResponse::PaymentViewResponse {
                            response: None,
                            transaction_id,
                            enterprise_id,
                            card_id,
                            initial_leader,
                        })
                }
            }
        }
    }
}

// impl Handler<PaymentView> for Repository {
//     type Result = ();
//     fn handle(&mut self, msg: PaymentView, _: &mut Context<Self>) {
//         if let Some(data) = self.cards.get(&msg.card_id) {
//             self.repository_manager.do_send(PaymentViewResponse {
//                 transaction_id: msg.transaction_id,
//                 enterprise_id: self.id,
//                 card_id: msg.card_id,
//                 data: Some(PaymentViewResponseData {
//                     card_limit: data.get_limit(),
//                     card_usage: data.get_current_usage(),
//                     enterprise_usage: self.current_usage,
//                     enterprise_limit: self.limit.clone(),
//                 }),
//             });
//         } else {
//             self.repository_manager.do_send(PaymentViewResponse {
//                 transaction_id: msg.transaction_id,
//                 enterprise_id: self.id,
//                 card_id: msg.card_id,
//                 data: None,
//             });
//         }
//     }
// }

impl Handler<EnterpriseLimitUpdate> for Repository {
    type Result = ();
    fn handle(&mut self, msg: EnterpriseLimitUpdate, _: &mut Context<Self>) {
        let mut limit = self.limit.unwrap_or(0);
        match msg.update_type {
            UpdateType::Increment => limit += msg.limit,
            UpdateType::Decrement => limit -= msg.limit,
            UpdateType::Set => limit = msg.limit,
        }
        //self.limit = Some(limit); // /SOLAMENTE SE ALMACENA EL DATO DE MANERA PROVISIONAL EN "SetOwnData"

        let set_own_data = SetOwnData {
            enterprise_id: self.id,
            data: UpdateRegister::EnterpriseLimit {
                limit,
                usage: self.current_usage,
            },
        };
        self.repository_manager.do_send(SendBroadcastOfSetOwnData {
            initial_leader: msg.initial_leader,
            command: set_own_data,
            result: true,
            do_broadcast: true,
        });
    }
}

/// Asumimos que si tenemos el repositorio, tenemos todas las tarjetas,
/// así que no hay necesidad de tirar error en este punto
impl Handler<CardLimitUpdate> for Repository {
    type Result = ();
    fn handle(&mut self, msg: CardLimitUpdate, _: &mut Context<Self>) {
        let reg = self.cards.get_mut(&msg.card_id).expect(&format!(
            "Card with id {} not found - From UpdatePayment at Repository",
            msg.card_id
        ));
        let Some(current_limit) = reg.get_limit() else {
            log!("[R {}] No current limit found in register", self.id);
            return;
        };
        let new_limit = match msg.update_type {
            UpdateType::Increment => current_limit + msg.limit,
            UpdateType::Decrement => current_limit - msg.limit,
            UpdateType::Set => msg.limit,
        };
        // match msg.update_type {                                      // /SOLAMENTE SE ALMACENA EL DATO DE MANERA PROVISIONAL EN "SetOwnData"
        //     UpdateType::Increment => reg.increment_limit(msg.limit),
        //     UpdateType::Decrement => reg.decrement_limit(msg.limit),
        //     UpdateType::Set => reg.set_limit(Some(msg.limit)),
        // }
        let initial_leader = msg.initial_leader;
        let mut new_register = SpendingInfoRegister::new(reg.get_card_id(), Some(new_limit));
        new_register.set_usage(reg.get_current_usage());

        Self::send_broadcast_to_set_own_data(
            &mut self.repository_manager,
            initial_leader,
            &mut new_register,
            self.id,
            self.limit.unwrap_or(0),
            self.current_usage,
            true,
            true,
        );
    }
}

///* Asumimos que si tenemos el repositorio, tenemos todas las tarjetas,
///así que no hay necesidad de tirar error en este punto
///* Se presume que siempre, tanto empresas como tarjetas tienen pre-establecido un límite
impl Handler<UpdatePayment> for Repository {
    type Result = ();
    fn handle(&mut self, msg: UpdatePayment, _ctx: &mut Context<Self>) {
        let reg = self.cards.get_mut(&msg.card_id).expect(&format!(
            "Card with id {} not found - From UpdatePayment at Repository",
            msg.id
        ));
        // Si el limite no existe lo setteo a el usage actual + el nuevo
        // para que siempre la comparacion de false
        let initial_leader = msg.initial_leader;
        let enterprise_limit = self.limit.unwrap_or(self.current_usage + msg.cost);
        let card_limit = reg
            .get_limit()
            .unwrap_or(reg.get_current_usage() + msg.cost);
        if (self.current_usage + msg.cost > enterprise_limit)
            || (reg.get_current_usage() + msg.cost > card_limit)
        {
            Self::send_broadcast_to_set_own_data(
                &mut self.repository_manager,
                initial_leader,
                reg,
                self.id,
                self.limit.unwrap_or(0),
                self.current_usage,
                false,
                false, // /NO SE REQUERIRÁ DE HACER BROADCAST DE ESTA OPERACIÓN, PUES SOBREPASA EL/LOS LÍMITE/S
            );
        } else {
            let new_enterprise_usage = self.current_usage + msg.cost;
            let mut new_register = SpendingInfoRegister::new(reg.get_card_id(), Some(card_limit));
            new_register.set_usage(reg.get_current_usage() + msg.cost);

            // reg.increase_usage(msg.cost);     // /SOLAMENTE SE ALMACENA EL DATO DE MANERA PROVISIONAL EN "SetOwnData"
            // self.current_usage += msg.cost;
            Self::send_broadcast_to_set_own_data(
                &mut self.repository_manager,
                initial_leader,
                &mut new_register,
                self.id,
                self.limit.unwrap_or(0),
                new_enterprise_usage,
                true,
                true,
            );
        }
    }
}

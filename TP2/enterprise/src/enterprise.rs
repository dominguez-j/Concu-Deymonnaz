use crate::enterprise_config::EnterpriseConfig;
use crate::logger::{LogEvent, Logger};
use crate::messages::{AdminCommand, SetProxyAddr};
use crate::proxy::Proxy;
use actix::{Actor, Addr, Context, Handler};
use lib::prelude::*;
use std::collections::HashMap;

pub struct Enterprise {
    id: u32,
    balance: u32,
    cards: HashMap<u32, u32>,
    logger: Logger,
    proxy_addr: Option<Addr<Proxy>>,
}

impl Enterprise {
    pub fn new(config: EnterpriseConfig) -> Self {
        let cards = config.cards.into_iter().map(|c| (c.id, c.limit)).collect();
        Enterprise {
            id: config.id,
            balance: config.balance,
            cards,
            logger: Logger,
            proxy_addr: None,
        }
    }
}

impl Actor for Enterprise {
    type Context = Context<Self>;
}

impl Handler<AdminCommand> for Enterprise {
    type Result = ();

    fn handle(&mut self, msg: AdminCommand, _ctx: &mut Context<Self>) {
        match msg {
            AdminCommand::UpdateEnterpriseLimit { limit, update_type } => {
                if let Some(addr) = &self.proxy_addr {
                    let msg = ProtocolMessage::Update(Update::EnterpriseLimitUpdate {
                        limit,
                        update_type,
                        enterprise_id: self.id,
                    });
                    addr.do_send(msg);
                }
            }
            AdminCommand::UpdateCardLimit {
                card_id,
                limit,
                update_type,
            } => {
                if self.cards.contains_key(&card_id) {
                    if let Some(addr) = &self.proxy_addr {
                        let msg = ProtocolMessage::Update(Update::CardLimitUpdate {
                            card_id,
                            limit,
                            update_type,
                            enterprise_id: self.id,
                        });
                        addr.do_send(msg);
                    }
                }
            }
            AdminCommand::CardView { card_id } => {
                if let Some(addr) = &self.proxy_addr {
                    let msg = ProtocolMessage::Select(Select::CardView {
                        enterprise_id: self.id,
                        card_id,
                    });
                    addr.do_send(msg);
                }
            }
            AdminCommand::EnterpriseView {} => {
                if let Some(addr) = &self.proxy_addr {
                    let msg = ProtocolMessage::Select(Select::EnterpriseView {
                        enterprise_id: self.id,
                    });
                    addr.do_send(msg);
                }
            }
        }
    }
}

impl Handler<SetProxyAddr> for Enterprise {
    type Result = ();

    fn handle(&mut self, msg: SetProxyAddr, _ctx: &mut Context<Self>) {
        self.proxy_addr = Some(msg.0);

        if let Some(addr) = &self.proxy_addr {
            let start_up_msg = ProtocolMessage::StartUp {
                from: self.id,
                role: Role::Proxy,
            };
            addr.do_send(start_up_msg);
            let initial_state_msg = ProtocolMessage::EnterpriseCreate {
                enterprise_id: self.id,
                enterprise_balance: self.balance,
                card_limits: self.cards.clone(),
            };
            addr.do_send(initial_state_msg);
        }
    }
}

impl Handler<ProtocolMessage> for Enterprise {
    type Result = ();

    fn handle(&mut self, msg: ProtocolMessage, _ctx: &mut Context<Self>) {
        match msg {
            ProtocolMessage::Update(Update::Payment {
                payment_id: id,
                enterprise_id: _,
                card_id,
                transaction_type: _,
                cost: amount,
            }) => {
                let event = LogEvent::Transaction {
                    card_id,
                    amount,
                    station_id: IdInterpreter::get_station_id(id).unwrap(),
                };
                self.logger.log(event);
            }
            ProtocolMessage::TransactionResponse(TransactionResponse::CardViewResponse {
                response,
                card_id,
                ..
            }) => {
                if let Some(info) = response {
                    let event = LogEvent::CardView {
                        card_id,
                        usage: info.card_usage,
                        limit: info.card_limit.unwrap_or(0),
                    };
                    self.logger.log(event);
                }
            }
            ProtocolMessage::TransactionResponse(TransactionResponse::EnterpriseViewResponse {
                response,
                ..
            }) => {
                if let Some(data) = response {
                    let event = LogEvent::EnterpriseView {
                        usage: data.usage,
                        limit: data.limit,
                    };
                    self.logger.log(event);
                }
                // TODO: Añadir un log en caso de que no llegue nada
            }
            ProtocolMessage::FirstMsg { from, data } => {
                let event = LogEvent::FirstMsg { from, data };
                self.logger.log(event);
            }
            _ => {
                println!("Enterprise received unknown message: {:?}", msg);
            }
        }
    }
}

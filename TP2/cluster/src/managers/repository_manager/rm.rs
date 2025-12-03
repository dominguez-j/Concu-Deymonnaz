use crate::TransactionManager;
use crate::managers::connection_manager::handlers::SetTransactionManagerAddr;
use crate::repository::Repository;
use actix::prelude::*;
use lib::messages::set_own_data::{SendBroadcastOfSetOwnData, SetOwnData};
use lib::prelude::*;
use std::collections::HashMap;

impl Handler<SetOwnData> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: SetOwnData, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(rep) = self.enterprises.get(&msg.enterprise_id) {
            rep.do_send(msg);
        }
        // NOTA: Esto sería si queremos que SetOwnData permita actualizar los datos
        //       cuando el nodo cayó y los perdió. Por ahora, no lo hacemos
        // else {
        //     let new_repository = Repository::new(msg.enterprise_id, ctx.address());
        //     let card_limits = HashMap::new();
        //     new_repository.do_send(Create {
        //         enterprise_id: msg.enterprise_id,
        //         enterprise_limit: 0,
        //         card_limits,
        //     });
        //     new_repository.do_send(msg);
        // }
    }
}

impl Handler<SendBroadcastOfSetOwnData> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: SendBroadcastOfSetOwnData, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(tm_addr) = &self.transaction_manager {
            tm_addr.do_send(msg);
        }
    }
}

pub struct RepositoryManager {
    pub enterprises: HashMap<u32, Addr<Repository>>, // Map<enterprise_id, Repository>
    pub transaction_manager: Option<Addr<TransactionManager>>,
}

impl RepositoryManager {
    pub fn new() -> Addr<Self> {
        Self {
            enterprises: HashMap::new(),
            transaction_manager: None,
        }
        .start()
    }
}

impl Actor for RepositoryManager {
    type Context = Context<Self>;
}

impl Handler<SetTransactionManagerAddr> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: SetTransactionManagerAddr, _: &mut Self::Context) -> Self::Result {
        self.transaction_manager = Some(msg.addr);
    }
}

impl Handler<Create> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: Create, ctx: &mut Self::Context) -> Self::Result {
        let repo = Repository::new(msg.enterprise_id, ctx.address());
        repo.do_send(msg.clone());
        self.enterprises.insert(msg.enterprise_id, repo);
    }
}

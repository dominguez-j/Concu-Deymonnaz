use crate::RepositoryManager;
use crate::messages::{CardViewResponse, EnterpriseViewResponse, PaymentViewResponse};
use actix::Handler;
use lib::messages::response::InternodeResponse;

impl Handler<CardViewResponse> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: CardViewResponse, _: &mut Self::Context) -> Self::Result {
        let tm = self
            .transaction_manager
            .clone()
            .expect("TransactionManager is none in CardViewResponse");
        if msg.response.is_none() {
            tm.do_send(msg.to_is());
        } else {
            tm.do_send(msg);
        }
    }
}

impl Handler<EnterpriseViewResponse> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: EnterpriseViewResponse, _: &mut Self::Context) -> Self::Result {
        if let Some(tm) = &self.transaction_manager {
            tm.do_send(msg);
        }
    }
}

impl Handler<InternodeResponse> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: InternodeResponse, _: &mut Self::Context) -> Self::Result {
        self.transaction_manager
            .as_mut()
            .expect("TransactionManager is none when InternodeResponse in RepositoryManager")
            .do_send(msg);
    }
}

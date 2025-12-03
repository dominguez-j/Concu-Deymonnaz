use crate::RepositoryManager;
use crate::messages::*;
use crate::transaction_with_id::*;
use actix::Handler;
use lib::prelude::*;

impl Handler<SelectWithId> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: SelectWithId, _: &mut Self::Context) -> Self::Result {
        let enterprise_id = msg.select.get_enterprise_id();
        let _tm = self
            .transaction_manager
            .clone()
            .expect("Missing TransactionManager when making a select");
        if let Some(rep) = self.enterprises.get_mut(&enterprise_id) {
            let select = msg.select;
            match select {
                Select::CardView { card_id, .. } => {
                    rep.do_send(CardView {
                        transaction_id: msg.transaction_id,
                        card_id,
                    });
                }
                Select::EnterpriseView { .. } => {
                    rep.do_send(EnterpriseView {
                        transaction_id: msg.transaction_id,
                    });
                }
            }
        } else {
            let tm = self
                .transaction_manager
                .clone()
                .expect("TransactionManager is none in SelectWithId");
            tm.do_send(msg.to_is());
        }
    }
}

impl Handler<InternodeSelectWithId> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: InternodeSelectWithId, _: &mut Self::Context) -> Self::Result {
        let tm = self
            .transaction_manager
            .clone()
            .expect("Missing TransactionManager when making an internode select");
        let cloned_message = msg.clone();
        if let Some(rep) = self
            .enterprises
            .get_mut(&cloned_message.select.get_enterprise_id())
        {
            rep.do_send(msg)
        } else {
            if let Some(response) = cloned_message.select.to_ir() {
                tm.do_send(response);
            }
        }
    }
}

///* Se verifica si se tiene el Repository para la respectiva transacción, con lo cual de contar
///con dicho Repository indicará que se tiene registro de todas las tarjetas referentes al mismo,
///caso contrario, se procederá a realizar una request internodo para solicitar la información a
///otro nodo que sí la posea, para de esta forma decidir si la operación Update es viable o no.
impl Handler<UpdateWithId> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: UpdateWithId, _: &mut Self::Context) -> Self::Result {
        let enterprise_id = msg.update.get_enterprise_id();

        if let Some(rep) = self.enterprises.get_mut(&enterprise_id) {
            let update = msg.update;
            let initial_leader = msg.initial_leader;
            match update {
                Update::EnterpriseLimitUpdate {
                    limit, update_type, ..
                } => {
                    rep.do_send(EnterpriseLimitUpdate {
                        initial_leader,
                        limit,
                        update_type,
                    });
                }
                Update::CardLimitUpdate {
                    limit,
                    card_id,
                    update_type,
                    ..
                } => {
                    rep.do_send(CardLimitUpdate {
                        initial_leader,
                        limit,
                        card_id,
                        update_type,
                    });
                }
                // TODO: Añadir el payment type acá para poder hacer el ForcePayment
                Update::Payment {
                    payment_id,
                    card_id,
                    cost,
                    ..
                } => {
                    rep.do_send(UpdatePayment {
                        initial_leader,
                        id: payment_id,
                        card_id,
                        cost,
                    });
                }
            }
        } else {
            let tm = self
                .transaction_manager
                .clone()
                .expect("TransactionManager is none in UpdateWithId");
            tm.do_send(msg.to_is());
        }
    }
}

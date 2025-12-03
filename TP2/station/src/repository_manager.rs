use crate::message::payments_map::SavedPayments;
use crate::message::{clear::Clear, get_all::GetAll, remove::Remove};
use crate::station::Station;
use actix::prelude::*;
use lib::prelude::*;
use tokio::io::{AsyncWriteExt, Lines};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncBufReadExt, BufReader},
};

pub struct RepositoryManager {
    station: Addr<Station>,
    repository_name: String,
    repository: Option<File>,
}

impl RepositoryManager {
    pub fn new(station: Addr<Station>, repository_name: String) -> Addr<Self> {
        Self {
            station,
            repository_name,
            repository: None,
        }
        .start()
    }
    async fn get_line_reader(repository_name: &str) -> Option<Lines<BufReader<File>>> {
        let file = match OpenOptions::new().read(true).open(repository_name).await {
            Ok(f) => f,
            Err(_) => return None,
        };
        let reader = BufReader::new(file);
        Some(reader.lines())
    }
    async fn get_truncated_writer(repository_name: &str) -> Option<File> {
        Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(repository_name)
                .await
                .unwrap(),
        )
    }
    async fn write_payment(writer: &mut File, payment: &Payment) {
        writer
            .write_all((payment.as_representation() + "\n").as_bytes())
            .await
            .unwrap()
    }
}

impl Actor for RepositoryManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let repo_name = self.repository_name.clone();
        ctx.wait(
            async move {
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(repo_name)
                    .await
                    .unwrap()
            }
            .into_actor(self)
            .map(|file, repository_manager, _| {
                repository_manager.repository = Some(file);
            }),
        )
    }
}

impl Handler<Payment> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: Payment, ctx: &mut Self::Context) -> Self::Result {
        log!("[RMSTATION] Persisting in RM");
        let mut repo = self.repository.take().unwrap();
        ctx.wait(
            async move {
                Self::write_payment(&mut repo, &msg).await;
                repo
            }
            .into_actor(self)
            .map(|repo, rmanager, _| {
                rmanager.repository = Some(repo);
            }),
        );
    }
}

impl Handler<Remove> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, msg: Remove, ctx: &mut Self::Context) -> Self::Result {
        let repo = self.repository_name.clone();
        ctx.wait(
            async move {
                let mut payments = Vec::new();
                let mut reader = RepositoryManager::get_line_reader(&repo).await.unwrap();
                let mut writer = RepositoryManager::get_truncated_writer(&repo)
                    .await
                    .unwrap();

                while let Some(line) = reader.next_line().await.unwrap() {
                    if let Ok(payment) = serde_json::from_str::<Payment>(&line) {
                        if payment.id() != msg.transaction_id() {
                            payments.push(payment);
                        }
                    }
                }
                for payment in payments {
                    Self::write_payment(&mut writer, &payment).await;
                }
                Some(writer)
            }
            .into_actor(self)
            .map(|writer, rmanager, _| {
                rmanager.repository = writer;
            }),
        )
    }
}

impl Handler<GetAll> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, _: GetAll, ctx: &mut Self::Context) -> Self::Result {
        let repo_name = self.repository_name.clone();
        let station = self.station.clone();
        ctx.wait(
            async move {
                let mut reader = Self::get_line_reader(&repo_name).await.unwrap();
                let mut result = Vec::new();
                while let Ok(Some(line)) = reader.next_line().await {
                    let payment = Payment::from_representation(line);
                    result.push(payment);
                }
                station.do_send(SavedPayments::new(result));
            }
            .into_actor(self),
        );
    }
}

impl Handler<Clear> for RepositoryManager {
    type Result = ();
    fn handle(&mut self, _: Clear, ctx: &mut Self::Context) -> Self::Result {
        println!("Clearing saved data");
        let repo_name = self.repository_name.clone();
        ctx.wait(
            async move { Self::get_truncated_writer(&repo_name).await.unwrap() }
                .into_actor(self)
                .map(|file, rmanager, _| {
                    rmanager.repository = Some(file);
                }),
        );
    }
}

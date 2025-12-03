mod config;
mod connection_establisher;
mod heartbeat_sender;
mod message;
mod pump_connection_listener;
mod repository_manager;
mod station;

use crate::config::Config;
use crate::station::Station;

#[actix::main]
async fn main() {
    let config = Config::default();
    let _ = Station::new(
        config.station_id(),
        config.repository_name().to_string(),
        config.station_udp_listening_port(),
        config.pumps_udp_listening_port(),
        config.cluster_address().to_string(),
        config.hostname().to_string(),
        config.base_port(),
    )
    .await;
    tokio::signal::ctrl_c().await.unwrap();
}

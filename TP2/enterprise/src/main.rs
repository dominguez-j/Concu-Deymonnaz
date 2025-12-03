mod admin;
mod enterprise;
mod enterprise_config;
mod logger;
mod messages;
mod proxy;
mod proxy_config;

use crate::admin::AdminInput;
use crate::enterprise::Enterprise;
use crate::enterprise_config::EnterpriseConfig;
use crate::messages::SetProxyAddr;
use crate::proxy::Proxy;
use crate::proxy_config::ProxyConfig;
use actix::Actor;
use std::env;

#[actix::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config_file>", args[0]);
        return;
    }
    let config_path = &args[1];

    let enterprise_cfg =
        EnterpriseConfig::from_file(config_path).expect("Could not load configuration.");
    let proxy_cfg = ProxyConfig::default();

    let enterprise = Enterprise::new(enterprise_cfg.clone());
    let enterprise_addr = Enterprise::start(enterprise);
    let proxy = Proxy::new(
        enterprise_addr.clone(),
        enterprise_cfg.id.clone(),
        proxy_cfg.host.clone(),
        proxy_cfg.proxy_port.clone(),
        proxy_cfg.internode_port.clone(),
        proxy_cfg.node_count.clone(),
    )
    .await;

    enterprise_addr.do_send(SetProxyAddr(proxy));
    AdminInput::new(enterprise_addr);
    tokio::signal::ctrl_c().await.unwrap();
}

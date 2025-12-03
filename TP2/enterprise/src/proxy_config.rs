use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub proxy_port: u32,
    pub internode_port: u32,
    pub node_count: u32,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ProxyConfig {
    fn from_env() -> Self {
        dotenv().ok();
        Self {
            host: env::var("HOST").unwrap().parse().unwrap(),
            proxy_port: env::var("PROXY_PORT_BASE").unwrap().parse().unwrap(),
            internode_port: env::var("INTERNODE_PORT_BASE").unwrap().parse().unwrap(),
            node_count: env::var("NUM_NODES").unwrap().parse().unwrap(),
        }
    }
}

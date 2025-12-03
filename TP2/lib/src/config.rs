use crate::logger::LogLevel;
use dotenv::dotenv;
use std::env;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Config {
    pub log_level: LogLevel,
    pub num_nodes: u32,
    pub discovery_rounds: u32,
    pub hb_misses_allowed: u32,
    pub server_port_base: u32,
    pub proxy_port_base: u32,
    pub station_port_base: u32,
    pub server_peer: Vec<u32>,
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn new() -> Config {
        dotenv().ok();

        let log_level = match env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "Info".to_string())
            .as_str()
        {
            "None" => LogLevel::None,
            "Trace" => LogLevel::Trace,
            "Debug" => LogLevel::Debug,
            "Info" => LogLevel::Info,
            "Warn" => LogLevel::Warn,
            "Error" => LogLevel::Error,
            _ => LogLevel::Info,
        };

        let server_port_base = env::var("C")
            .unwrap_or_else(|_| "7000".to_string())
            .parse::<u32>()
            .unwrap();

        let proxy_port_base = env::var("PROXY_PORT_BASE")
            .unwrap_or_else(|_| "6000".to_string())
            .parse::<u32>()
            .unwrap();

        let station_port_base = env::var("STATION_PORT_BASE")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u32>()
            .unwrap();

        let num_nodes = env::var("NUM_NODES")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u32>()
            .unwrap();

        let discovery_rounds = env::var("MAX_DISCOVERY_ROUNDS")
            .unwrap_or_else(|_| "2".to_string())
            .parse::<u32>()
            .unwrap();

        let hb_misses_allowed = env::var("HB_MISSES_ALLOWED")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u32>()
            .unwrap();

        let server_peer = (0..num_nodes)
            .map(|x| x + (server_port_base + 1))
            .collect::<Vec<_>>();

        Config {
            log_level,
            num_nodes,
            discovery_rounds,
            hb_misses_allowed,
            server_port_base,
            proxy_port_base,
            station_port_base,
            server_peer,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

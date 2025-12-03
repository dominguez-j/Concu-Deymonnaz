use dotenvy::dotenv;
use std::env;
use std::str::FromStr;

pub struct Config {
    station_id: u32,
    repository_name: String,
    station_udp_listening_port: u32,
    pumps_udp_listening_port: u32,
    cluster_address: String,
    hostname: String,
    base_port: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config::from_env()
    }
}

impl Config {
    fn from_env() -> Self {
        dotenv().ok();
        let station_id: u32 = Config::load("STATION_ID");
        let repository_name: String = Config::load("REPOSITORY_NAME");
        let station_udp_listening_port: u32 = Config::load("STATION_UDP_LISTENING_PORT");
        let pumps_udp_listening_port: u32 = Config::load("PUMPS_UDP_LISTENING_PORT");
        let cluster_address: String = Config::load("CLUSTER_ADDRESS");
        let hostname: String = Config::load("HOSTNAME");
        let base_port: u32 = Config::load("BASE_PORT");
        Self {
            station_id,
            repository_name,
            station_udp_listening_port,
            pumps_udp_listening_port,
            cluster_address,
            hostname,
            base_port,
        }
    }
    fn load<T>(key: &str) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        env::var(key)
            .expect(&format!("Key {} missing", key))
            .parse()
            .expect(&format!("Invalid {} key", key))
    }
    pub fn station_id(&self) -> u32 {
        self.station_id
    }
    pub fn repository_name(&self) -> &str {
        &self.repository_name
    }
    pub fn station_udp_listening_port(&self) -> u32 {
        self.station_udp_listening_port
    }
    pub fn pumps_udp_listening_port(&self) -> u32 {
        self.pumps_udp_listening_port
    }
    pub fn cluster_address(&self) -> &str {
        &self.cluster_address
    }
    pub fn hostname(&self) -> &str {
        &self.hostname
    }
    pub fn base_port(&self) -> u32 {
        self.base_port
    }
}

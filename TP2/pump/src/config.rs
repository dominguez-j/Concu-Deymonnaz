use dotenvy::dotenv;
use std::env;
use std::str::FromStr;

pub struct Config {
    station_address: String,
    ping_receive_base_port: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config::from_env()
    }
}

impl Config {
    fn from_env() -> Self {
        dotenv().ok();
        let station_address: String = Config::load("STATION_ADDRESS");
        let ping_receive_base_port: u32 = Config::load("PING_RECEIVE_BASE_PORT");
        Self {
            station_address,
            ping_receive_base_port,
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
    pub fn station_address(&self) -> &str {
        &self.station_address
    }
    pub fn ping_base_receive_port(&self) -> u32 {
        self.ping_receive_base_port
    }
}

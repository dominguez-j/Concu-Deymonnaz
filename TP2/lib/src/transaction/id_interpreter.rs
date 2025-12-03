use std::io::{Error, ErrorKind, Result};

pub struct IdInterpreter;

impl IdInterpreter {
    fn count_separators(id: &str) -> usize {
        id.chars()
            .fold(0, |acc, c| acc + if c == '-' { 1 } else { 0 })
    }
    pub fn build_internode_request_id(number: u32) -> String {
        format!("internode+{}", number)
    }
    pub fn build_payment_id(enterprise: u32, card: u32, pump: u32, station: u32) -> String {
        format!("{}-{}-{}-{}", enterprise, card, pump, station)
    }
    pub fn build_spending_update_id(enterprise: u32, card: u32) -> String {
        format!("{}-{}", enterprise, card)
    }
    pub fn build_enterprise_limit_update_id(enterprise: u32) -> String {
        format!("{}", enterprise)
    }
    pub fn check_internode_id(id: &str) -> bool {
        id.splitn(3, '*').collect::<Vec<&str>>()[1] == "internode"
    }
    pub fn get_enterprise_id(id: String) -> u32 {
        if let Ok(res) = id.splitn(2, '-').collect::<Vec<&str>>()[0].parse() {
            return res;
        } else if let Ok(res) = id.splitn(2, '+').collect::<Vec<&str>>()[0].parse() {
            return res;
        }
        panic!("[ID] Wrong id format: {}", id);
    }
    pub fn get_card_id(id: String) -> Result<u32> {
        if Self::count_separators(&id) != 2 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid type of id"));
        }
        id.splitn(3, '-').collect::<Vec<&str>>()[1]
            .parse::<u32>()
            .or(Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid type of id",
            )))
    }
    pub fn get_pump_id(id: String) -> Result<u32> {
        if Self::count_separators(&id) != 3 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid type of id"));
        }
        Ok(id.splitn(4, '-').collect::<Vec<&str>>()[2].parse().unwrap())
    }
    pub fn get_station_id(id: String) -> Result<u32> {
        if Self::count_separators(&id) != 3 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid type of id"));
        }
        Ok(id.split_once('-').unwrap().1.parse().unwrap())
    }
}

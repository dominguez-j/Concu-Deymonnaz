#[derive(Debug)]
pub enum LogEvent {
    Transaction {
        card_id: u32,
        amount: u32,
        station_id: u32,
    },
    CardView {
        card_id: u32,
        usage: u32,
        limit: u32,
    },
    EnterpriseView {
        usage: u32,
        limit: u32,
    },
    FirstMsg {
        from: u32,
        data: String,
    },
}

pub struct Logger;

impl Logger {
    pub fn log(&self, event: LogEvent) {
        match event {
            LogEvent::Transaction {
                card_id,
                amount,
                station_id,
            } => {
                println!(
                    "[PROXY] Card {} - Amount {} - Station {}",
                    card_id, amount, station_id
                );
            }
            LogEvent::CardView {
                card_id,
                usage,
                limit,
            } => {
                println!(
                    "[ADMIN] Card {} - Usage: {} - Limit: {}",
                    card_id, usage, limit
                );
            }
            LogEvent::EnterpriseView { usage, limit } => {
                println!("[ADMIN] Enterprise - Usage: {} - Limit: {}", usage, limit);
            }
            LogEvent::FirstMsg { from, data } => {
                //println!("[FIRST_MSG] Node {} - Data: {}", from, data);
            }
        }
    }
}

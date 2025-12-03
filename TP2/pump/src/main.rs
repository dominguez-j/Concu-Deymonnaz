use crate::config::Config;
use crate::pump::Pump;
use lib::transaction::card::Card;
use lib::transaction::payment::Payment;
use std::time::Duration;

mod config;
mod connection_establisher;
mod pump;

#[actix::main]
async fn main() {
    let config = Config::default();
    let pump_1 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let pump_2 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let pump_3 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let pump_4 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let addr = config.station_address();
    let port = addr.splitn(2, ':').collect::<Vec<&str>>()[1]
        .parse::<u16>()
        .unwrap()
        + 1;
    let new_addr = format!("{}:{}", addr.splitn(2, ':').collect::<Vec<&str>>()[0], port);

    let ping_port = config.ping_base_receive_port() + 1000;
    let pump_5 = Pump::new(new_addr.clone(), ping_port);
    let pump_6 = Pump::new(new_addr.clone(), ping_port);
    let pump_7 = Pump::new(new_addr.clone(), ping_port);
    let pump_8 = Pump::new(new_addr.clone(), ping_port);

    let pump_9 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let pump_10 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let pump_11 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );
    let pump_12 = Pump::new(
        config.station_address().to_string(),
        config.ping_base_receive_port(),
    );

    let pump_13 = Pump::new(new_addr.clone(), ping_port);
    let pump_14 = Pump::new(new_addr.clone(), ping_port);
    let pump_15 = Pump::new(new_addr.clone(), ping_port);
    let pump_16 = Pump::new(new_addr.clone(), ping_port);

    tokio::time::sleep(Duration::from_secs(5)).await;
    let card_1 = Card::new(1, 1);
    let card_2 = Card::new(2, 1);
    let card_3 = Card::new(3, 2);
    let card_4 = Card::new(4, 2);

    actix::spawn(async move {
        for i in 0..30 {
            // STATION 1
            pump_1.do_send(Payment::new(&card_1, 10));
            pump_2.do_send(Payment::new(&card_2, 10));
            pump_3.do_send(Payment::new(&card_1, 10));
            pump_4.do_send(Payment::new(&card_2, 10));

            // STATION 2
            // pump_5.do_send(Payment::new(&card_1, 10));
            // pump_6.do_send(Payment::new(&card_2, 10));
            // pump_7.do_send(Payment::new(&card_1, 10));
            // pump_8.do_send(Payment::new(&card_2, 10));

            // STATION 1
            pump_9.do_send(Payment::new(&card_3, 10));
            pump_10.do_send(Payment::new(&card_4, 10));
            pump_11.do_send(Payment::new(&card_3, 10));
            pump_12.do_send(Payment::new(&card_4, 10));

            // STATION 2
            // pump_13.do_send(Payment::new(&card_3, 10));
            // pump_14.do_send(Payment::new(&card_4, 10));
            // pump_15.do_send(Payment::new(&card_3, 10));
            // pump_16.do_send(Payment::new(&card_4, 10));
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
}

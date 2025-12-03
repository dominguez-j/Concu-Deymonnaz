use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ipc::representable::Representable;
use crate::transaction::card::Card;

#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct Payment {
    // These fields will be filled when passing the
    // payment to the next step of processing
    id: String,
    pump_id: Option<u32>,
    station_id: Option<u32>,

    // These fields will be provided by the initial
    // message
    card_id: u32,
    enterprise_id: u32,
    cost: u32, // ARS
}

impl Payment {
    pub fn new(card: &Card, cost: u32) -> Self {
        Self {
            id: String::new(),
            pump_id: None,
            station_id: None,
            card_id: card.id(),
            enterprise_id: card.enterprise_id(),
            cost,
        }
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn pump_id(&self) -> Option<u32> {
        self.pump_id
    }
    pub fn station_id(&self) -> Option<u32> {
        self.station_id
    }
    pub fn card_id(&self) -> u32 {
        self.card_id
    }
    pub fn enterprise_id(&self) -> u32 {
        self.enterprise_id
    }
    pub fn cost(&self) -> u32 {
        self.cost
    }
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
    pub fn set_pump_id(&mut self, pump_id: u32) {
        self.pump_id = Some(pump_id);
    }
    pub fn set_station_id(&mut self, station_id: u32) {
        self.station_id = Some(station_id);
    }
}

impl Representable for Payment {}

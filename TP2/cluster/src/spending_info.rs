use actix::Message;

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct SpendingInfoRegister {
    card_id: u32,
    current_usage: u32,
    limit: Option<u32>,
}

impl SpendingInfoRegister {
    pub fn new(card_id: u32, limit: Option<u32>) -> Self {
        Self {
            card_id,
            current_usage: 0,
            limit,
        }
    }
    pub fn increase_usage(&mut self, inc: u32) {
        self.current_usage += inc;
    }
    pub fn set_usage(&mut self, usage: u32) {
        self.current_usage = usage;
    }
    pub fn increment_limit(&mut self, inc: u32) {
        self.limit = Some(self.limit.unwrap_or(0) + inc);
    }
    pub fn decrement_limit(&mut self, dec: u32) {
        self.limit = Some(self.limit.unwrap_or(0) - dec);
    }
    pub fn set_limit(&mut self, new_limit: Option<u32>) {
        self.limit = new_limit;
    }
    pub fn get_card_id(&self) -> u32 {
        self.card_id
    }
    pub fn get_current_usage(&self) -> u32 {
        self.current_usage
    }
    pub fn get_limit(&self) -> Option<u32> {
        self.limit
    }
}

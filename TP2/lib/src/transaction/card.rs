pub struct Card {
    id: u32,
    enterprise_id: u32,
}

impl Card {
    pub fn new(id: u32, enterprise_id: u32) -> Self {
        Self { id, enterprise_id }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn enterprise_id(&self) -> u32 {
        self.enterprise_id
    }
}

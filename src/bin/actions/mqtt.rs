pub struct Action {
    pub min: u32,
    pub max: u32,
    pub dial_required: bool,
    pub topic: &'static str
}

impl Action {
    pub const fn new(topic: &'static str) -> Self {
        Self {
            min: 10,
            max: 100,
            dial_required: false,
            topic,
        }
    }

    pub fn map_value(&self, value: u8) -> u8 {
        let range = self.max - self.min;
        let mapped_value = ((value as u32 * range) / 10) + self.min;
        mapped_value as u8
    }
}

pub struct Action {
    pub min: u32,
    pub max: u32,
    pub dial_required: bool,
    pub topic: &'static str
}

impl Action {
    pub const fn new(topic: &'static str) -> Self {
        Self {
            min: 0,
            max: 100,
            dial_required: false,
            topic,
        }
    }
}

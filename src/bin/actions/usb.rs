pub enum TriggerType {
    Toggle,
    Press,
    Hold,
}

pub struct Action {
    pub trigger: TriggerType,
    pub keycode: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    Press,
    Toggle,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action {
    pub trigger: TriggerType,
    pub keycode: u8,
}
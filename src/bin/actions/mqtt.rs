pub struct DialOptions {
    pub required: bool,
}

pub struct MapRangeOptions {
    pub required: bool,

    pub out_min: i32,
    pub out_max: i32,
}

impl MapRangeOptions {
    pub fn map_value(&self, value: u8) -> i32 {
        let in_min = 1;
        let in_max = 10;

        let in_range = (in_max - in_min) as i64;
        let out_range = (self.out_max - self.out_min) as i64;
        let value_offset = (value - in_min) as i64;

        let mapped = value_offset * out_range / in_range + self.out_min as i64;
        mapped as i32
    }
}

pub struct MapValuesOptions {
    pub required: bool,
    pub values: &'static [(u32, &'static str)],
}

pub enum DialMode {
    None,
    Normal(DialOptions),
    MapRange(MapRangeOptions),
    MapValues(MapValuesOptions),
}

pub struct Action {
    pub dial: DialMode,
    pub topic: &'static str,
}

impl Action {
    pub const fn new(topic: &'static str) -> Self {
        Self {
            dial: DialMode::Normal(DialOptions { required: false }),
            topic,
        }
    }

    pub const fn new_lamp_control(topic: &'static str) -> Self {
        Self {
            dial: DialMode::MapRange(MapRangeOptions {
                required: false,
                out_min: 10,
                out_max: 100,
            }),
            topic,
        }
    }
}

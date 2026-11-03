pub type TimestampNS = i64;
pub type Name = String;
pub type Value = f64;

#[derive(Clone, Debug)]
pub struct Metric {
    pub name: Name,
    pub value: Value,
    pub at: TimestampNS,
}

impl Metric {
    pub fn new(name: Name, value: Value, at: Option<TimestampNS>) -> Self {
        let at = at.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as TimestampNS)
                .unwrap_or(0)
        });
        Self { name, value, at }
    }
}

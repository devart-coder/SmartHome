// 192.168.0.100
//
pub const BROKER_URL: &str = "mqtt://192.168.0.102:1883";
pub const SDDI: &str = "TP-Link_D88D";
pub const PASSWORD: &str = "14236679";
pub enum LightSubscriptions {
    ControllRoom,
    Restroom,
    Bathroom,
    Breakroom,
}
impl LightSubscriptions {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ControllRoom => "controll_center",
            Self::Restroom => "restroom",
            Self::Bathroom => "bathroom",
            Self::Breakroom => "breakroom",
        }
    }
    pub fn path(&self) -> String {
        match self {
            _ => format!("smart_home/light_controll/{}", self.name()),
        }
    }
    pub fn with_ownership_suffix(&self) -> String {
        format!("{}/light_ownership", self.path())
    }
}

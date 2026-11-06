// 192.168.0.100
//
pub const BROKER_URL: &str = "mqtt://192.168.0.100:1883";
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
            Self::ControllRoom => "controllroom",
            Self::Restroom => "restroom",
            Self::Bathroom => "bathroom",
            Self::Breakroom => "breakroom",
        }
    }
    pub fn topic(&self) -> String {
        match self {
            _ => format!("smart_home/light_controll/{}", self.name()),
        }
    }
    pub fn topic_with_ownership(&self) -> String {
        format!("{}/light_ownership", self.topic())
    }
}

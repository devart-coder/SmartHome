use crate::subscriptions;

// 192.168.0.100
//
pub const BROKER_URL: &str = "mqtt://192.168.0.100:1883";
pub const SDDI: &str = "TP-Link_D88D";
pub const PASSWORD: &str = "14236679";
subscriptions!(ControllRoom, Restroom, Bathroom, Breakroom,);

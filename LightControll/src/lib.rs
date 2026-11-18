pub mod mqtt_handler;
pub mod relay;
pub mod wifi;
pub mod security {
    include!(concat!(env!("OUT_DIR"), "/security.rs"));
}

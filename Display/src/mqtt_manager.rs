use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use esp_idf_svc::mqtt::client::{self, EspMqttClient, MqttClientConfiguration};

use crate::security::BROKER_URL;

pub struct MqttClientManager {
    client: Arc<Mutex<Option<EspMqttClient<'static>>>>,
    url: &'static str,
    config: MqttClientConfiguration<'static>,
}
impl MqttClientManager {
    pub fn new(url: &'static str, config: MqttClientConfiguration<'static>) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            config,
            url,
        }
    }
    // pub fn execute(){};
    pub fn restart() {}
    pub fn start(&self) {
        // {
        //     *self.client.lock().unwrap() = match EspMqttClient::new(self.url, &self.config) {
        //         Ok(v) => Some(v.0),
        //         Err(e) => None,
        //     };
        // }
        // let client_guard = Arc::new(Mutex::new(client));
        // let c = client_guard.clone();
        // let connection_handle = move || {
        //     while let Ok(event) = connection.next() {
        //         match event.payload() {
        //             EventPayload::Connected(_) => {
        //                 info!("Успешно подключились к MQTT-брокеру!");
        //                 if let Err(e) = tx.send("connected") {
        //                     error!("[SendError]: {}", e);
        //                 }
        //             }
        //             EventPayload::Disconnected => {
        //                 warn!("Потеряно соединение с MQTT-брокером. Переподключение...");
        //                 if let Err(e) = tx.send("disconnected") {
        //                     error!("[SendError]: {}", e);
        //                 }
        //             }
        //             EventPayload::Error(e) => {
        //                 log::error!("Error: {e}");
        //                 if let Err(e) = tx.send("disconnected") {
        //                     error!("[SendError]: {e}");
        //                 }
        //             }
        //             EventPayload::Received {
        //                 id,
        //                 topic,
        //                 data,
        //                 details,
        //             } => {
        //                 info!("Received: {id} {} {:?} {:?}", topic.unwrap(), data, details);
        //             }
        //             EventPayload::Deleted(v) => {
        //                 info!("Deleted: {v}");
        //             }
        //             EventPayload::Published(v) => {
        //                 info!("Published: {v}");
        //             }
        //             EventPayload::Subscribed(v) => {
        //                 info!("Syubscribed: {v}");
        //             }
        //             _ => {}
        //         }
        //     }
        // };
        // thread::spawn(connection_handle);
        // let mrx_clone = mrx.clone();
        // let subscribe = move || {
        //     let ui_weak = ui.clone();
        //     while let Ok(v) = mrx_clone.lock().unwrap().recv() {
        //         if v == "connected" {
        //             ui_weak
        //                 .unwrap()
        //                 .global::<GlobalConnectionsStatus>()
        //                 .set_mqtt_status(v.into());
        //             let topics = vec![
        //                 LightSubscriptions::ControllRoom.topic(),
        //                 LightSubscriptions::ControllRoom.topic_with_ownership(),
        //                 LightSubscriptions::Restroom.topic(),
        //                 LightSubscriptions::Restroom.topic_with_ownership(),
        //                 LightSubscriptions::Bathroom.topic(),
        //                 LightSubscriptions::Bathroom.topic_with_ownership(),
        //                 LightSubscriptions::Breakroom.topic(),
        //                 LightSubscriptions::Breakroom.topic_with_ownership(),
        //             ];
        //             for topic in topics {
        //                 let result = c
        //                     .lock()
        //                     .unwrap()
        //                     .subscribe(topic.as_str(), QoS::AtLeastOnce);
        //                 match result {
        //                     Err(e) => {
        //                         error!("Не удалось подписаться на топик {}: {:?}", topic, e)
        //                     }
        //                     Ok(_) => info!("Успешная подписка на топик: {}", topic),
        //                 }
        //             }
        //         } else if v == "disconnected" {
        //             ui_weak
        //                 .unwrap()
        //                 .global::<GlobalConnectionsStatus>()
        //                 .set_mqtt_status("disconnected".into());
        //         } else if v == "connecting" {
        //             ui_weak
        //                 .clone()
        //                 .unwrap()
        //                 .global::<GlobalConnectionsStatus>()
        //                 .set_mqtt_status(v.into());
        //         }
        //     }
        // };
        // thread::spawn(subscribe);
    }
    pub fn disconnect() {}
}

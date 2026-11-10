use std::{
    sync::{Arc, Mutex},
    thread,
};

use esp_idf_svc::mqtt::client::{
    EspMqttClient, EspMqttConnection, EventPayload, MqttClientConfiguration, QoS,
};
use log::{info, warn};

use crate::security::LightSubscriptions;

pub struct MqttClientManager {
    client: Arc<Mutex<Option<EspMqttClient<'static>>>>,
    connection: Arc<Mutex<Option<EspMqttConnection>>>,
}
impl MqttClientManager {
    pub fn new(url: &'static str, config: &MqttClientConfiguration<'static>) -> Self {
        let s = Self {
            client: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(None)),
        };
        match EspMqttClient::new(url, config) {
            Ok((cli, con)) => {
                *s.connection.lock().unwrap() = Some(con);
                *s.client.lock().unwrap() = Some(cli);
            }
            Err(_) => {
                *s.connection.lock().unwrap() = None;
                *s.client.lock().unwrap() = None;
            }
        };
        s
    }

    pub fn restart() {}
    pub fn execute(&self) {
        let con_guard = self.connection.clone();
        let cli_guard = self.client.clone();
        thread::spawn(move || {
            let mut counter = 0;
            let mut con = match con_guard.lock().unwrap().take() {
                Some(c) => c,
                None => {
                    log::error!("Соединение уже запущено или не инициализировано");
                    return;
                }
            };
            while let Ok(event) = con.next() {
                match event.payload() {
                    EventPayload::Connected(_) => {
                        let cli_guard = cli_guard.clone();
                        info!("Успешно подключились к MQTT-брокеру!");
                        counter = 0;
                        thread::spawn(move || {
                            let topics = vec![
                                LightSubscriptions::ControllRoom.topic(),
                                LightSubscriptions::ControllRoom.topic_with_ownership(),
                                LightSubscriptions::Restroom.topic(),
                                LightSubscriptions::Restroom.topic_with_ownership(),
                                LightSubscriptions::Bathroom.topic(),
                                LightSubscriptions::Bathroom.topic_with_ownership(),
                                LightSubscriptions::Breakroom.topic(),
                                LightSubscriptions::Breakroom.topic_with_ownership(),
                            ];
                            for topic in topics {
                                match cli_guard.lock().unwrap().as_mut() {
                                    Some(c) => {
                                        let result = c.subscribe(topic.as_str(), QoS::AtLeastOnce);
                                        match result {
                                            Err(e) => {
                                                log::error!(
                                                    "Не удалось подписаться на топик {topic}: {:?}",
                                                    e
                                                )
                                            }
                                            Ok(_) => info!("Успешная подписка на топик: {topic}"),
                                        }
                                    }
                                    None => {}
                                };
                            }
                        });
                    }
                    EventPayload::Disconnected => {
                        counter += 1;
                        warn!("Потеряно соединение с MQTT-брокером. Переподключение...");
                        info!("Попытка {counter}...");
                        if counter >= 5 {
                            log::error!("Сбой соединения");
                            *cli_guard.lock().unwrap() = None;
                            return;
                        }
                    }
                    EventPayload::Error(e) => {
                        log::error!("Error: {e}");
                    }
                    _ => {}
                }
            }
        });
    }
    // pub fn handler(&self) {}
    // let client_guard = Arc::new(Mutex::new(client));
    // let c = client_guard.clone();
    // let mrx_clone = mrx.clone();
    // let subscribe = ;
    // thread::spawn(move || {
    //     let topics = vec![
    //         LightSubscriptions::ControllRoom.topic(),
    //         LightSubscriptions::ControllRoom.topic_with_ownership(),
    //         LightSubscriptions::Restroom.topic(),
    //         LightSubscriptions::Restroom.topic_with_ownership(),
    //         LightSubscriptions::Bathroom.topic(),
    //         LightSubscriptions::Bathroom.topic_with_ownership(),
    //         LightSubscriptions::Breakroom.topic(),
    //         LightSubscriptions::Breakroom.topic_with_ownership(),
    //     ];
    //     for topic in topics {
    //         let mut client = self.client.lock().unwrap().unwrap();
    //         let result = client.subscribe(topic.as_str(), QoS::AtLeastOnce);
    //         match result {
    //             Err(e) => {
    //                 log::error!("Не удалось подписаться на топик {topic}: {:?}", e)
    //             }
    //             Ok(_) => info!("Успешная подписка на топик: {topic}"),
    //         }
    //     }
    // });
    // }
    // pub fn disconnect(&self) {
    // *self.lock().unwrap() = None;
    // }
}

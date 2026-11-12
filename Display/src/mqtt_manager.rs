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
    url: &'static str,
    config: MqttClientConfiguration<'static>,
}
impl MqttClientManager {
    pub fn new(url: &'static str, config: MqttClientConfiguration<'static>) -> Self {
        let s = Self {
            client: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(None)),
            url,
            config,
        };
        match EspMqttClient::new(url, &s.config) {
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
    pub fn guard_client(&self) -> Arc<Mutex<Option<EspMqttClient<'static>>>> {
        self.client.clone()
    }
    pub fn restart(&mut self) {
        match EspMqttClient::new(self.url, &self.config) {
            Ok((cli, con)) => {
                *self.connection.lock().unwrap() = Some(con);
                *self.client.lock().unwrap() = Some(cli);
            }
            Err(_) => {
                *self.connection.lock().unwrap() = None;
                *self.client.lock().unwrap() = None;
            }
        };
        self.run();
        info!("Перезагрузка клиента...");
    }
    pub fn run(&self) {
        let con_guard = self.connection.clone();
        let cli_guard = self.client.clone();
        let connection_handle = move || {
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
                        let subscribe = move || {
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
                        };
                        thread::spawn(subscribe);
                    }
                    EventPayload::Disconnected => {
                        counter += 1;
                        warn!("Потеряно соединение с MQTT-брокером");
                        info!("Попытка переподключения {counter}...");
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
        };
        thread::spawn(connection_handle);
    }
}

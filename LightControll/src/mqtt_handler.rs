use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, warn};
use mqttrs::{Connack, ConnectReturnCode, Packet};

use crate::{relay::Relay, security::LightSubscriptions};

pub struct Handler {
    map: HashMap<String, Box<dyn Fn(String, Arc<Mutex<Relay>>)>>,
}
impl Handler {
    pub fn new() -> Self {
        let mut mp = HashMap::<String, Box<dyn Fn(String, Arc<Mutex<Relay>>)>>::new();
        mp.insert(
            LightSubscriptions::ControllRoom.topic(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    info!("Свет в центре управления включен");
                    relay_guard.modify(|r| r.ch7().turn_on());
                } else if payload == "0" || payload == "off" {
                    info!("Свет в центре управления выключен");
                    relay_guard.modify(|r| r.ch7().turn_off());
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        mp.insert(
            LightSubscriptions::ControllRoom.topic_with_ownership(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    info!("take ownership");
                    relay_guard.modify(|r| r.ch8().turn_on());
                } else if payload == "0" || payload == "off" {
                    info!("release ownership");
                    relay_guard.modify(|r| r.ch8().turn_off());
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        //--------------------------------------------------------------
        mp.insert(
            LightSubscriptions::Restroom.topic(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    info!("Свет в уборной включен");
                    relay_guard.modify(|r| r.ch5().turn_on());
                } else if payload == "0" || payload == "off" {
                    info!("Свет в уборной выключен");
                    relay_guard.modify(|r| r.ch5().turn_off());
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        mp.insert(
            LightSubscriptions::Restroom.topic_with_ownership(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    info!("take ownership");
                    relay_guard.modify(|r| r.ch6().turn_on());
                } else if payload == "0" || payload == "off" {
                    info!("release ownership");
                    relay_guard.modify(|r| r.ch6().turn_off());
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        //--------------------------------------------------------------
        mp.insert(
            LightSubscriptions::Bathroom.topic(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    relay_guard.modify(|r| r.ch3().turn_on());
                    info!("Свет в ванной включен");
                } else if payload == "0" || payload == "off" {
                    relay_guard.modify(|r| r.ch3().turn_off());
                    info!("Свет в ванной выключен");
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        mp.insert(
            LightSubscriptions::Bathroom.topic_with_ownership(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    info!("take ownership");
                    relay_guard.modify(|r| r.ch4().turn_on());
                } else if payload == "0" || payload == "off" {
                    info!("release ownership");
                    relay_guard.modify(|r| r.ch4().turn_off());
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        //--------------------------------------------------------------
        mp.insert(
            LightSubscriptions::Breakroom.topic(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    relay_guard.modify(|r| r.ch1().turn_on());
                    info!("Свет в комнате отдыха включен");
                } else if payload == "0" || payload == "off" {
                    relay_guard.modify(|r| r.ch1().turn_off());
                    info!("Свет в комнате отдыха выключен");
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        mp.insert(
            LightSubscriptions::Breakroom.topic_with_ownership(),
            Box::new(|payload, relay| {
                let mut relay_guard = relay.lock().unwrap();
                if payload == "1" || payload == "on" {
                    info!("take ownership");
                    relay_guard.modify(|r| r.ch2().turn_on());
                } else if payload == "0" || payload == "off" {
                    info!("release ownership");
                    relay_guard.modify(|r| r.ch2().turn_off());
                } else {
                    log::error!("Неизвестное значение {}", payload);
                }
            }),
        );
        //--------------------------------------------------------------
        Self { map: mp }
    }
    pub fn handle_client(
        &self,
        mut stream: TcpStream,
        relay: Arc<Mutex<Relay>>,
    ) -> anyhow::Result<()> {
        let client_addr = stream.peer_addr()?;
        info!("Подключился новый клиент: {:?}", client_addr);
        let mut buffer = Box::new([0xffu8; 1024]);
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        loop {
            buffer.fill(0x00);
            match stream.read(&mut *buffer) {
                Ok(0) => {
                    warn!("Клиент {:?} закрыл соединение.", client_addr);
                    break;
                }
                Ok(bytes_read) => {
                    let mut data_slice = &buffer[..bytes_read];
                    match mqttrs::decode_slice(&mut data_slice) {
                        Ok(Some(Packet::Connect(connect_packet))) => {
                            info!("MQTT CONNECT от: {}", connect_packet.client_id);
                            let connack = Packet::Connack(Connack {
                                session_present: false,
                                code: ConnectReturnCode::Accepted,
                            });
                            let mut out_buf = [0u8; 128];
                            if let Ok(len) = mqttrs::encode_slice(&connack, &mut out_buf) {
                                stream.write_all(&out_buf[..len])?;
                                stream.flush()?;
                                info!("Клиент успешно авторизован!");
                            }
                        }
                        Ok(Some(Packet::Pingreq)) => {
                            let pingresp = Packet::Pingresp;
                            let mut out_buf = [0u8; 16];
                            if let Ok(len) = mqttrs::encode_slice(&pingresp, &mut out_buf) {
                                let _ = stream.write_all(&out_buf[..len]);
                                let _ = stream.flush();
                            }
                        }
                        Ok(Some(Packet::Publish(pub_packet))) => {
                            let payload = String::from_utf8_lossy(&pub_packet.payload)
                                .to_string()
                                .to_lowercase();
                            if self.map.contains_key(pub_packet.topic_name) {
                                let f = self.map.get(pub_packet.topic_name).unwrap();
                                f(payload.clone(), relay.clone());
                            } else {
                                log::error!("Unknown topic: {}", pub_packet.topic_name);
                            }
                            match pub_packet.qospid {
                                mqttrs::QosPid::AtLeastOnce(pid) => {
                                    let puback = Packet::Puback(pid);
                                    let mut out_buf = [0u8; 64];
                                    if let Ok(len) = mqttrs::encode_slice(&puback, &mut out_buf) {
                                        stream.write_all(&out_buf[..len])?;
                                        stream.flush()?;
                                    }
                                    let response_packet = Packet::Publish(mqttrs::Publish {
                                        dup: false,
                                        qospid: mqttrs::QosPid::AtMostOnce, // Меняем на QoS 0 для ответа телефону
                                        retain: false,
                                        topic_name: pub_packet.topic_name, // Тот же топик, который ждет телефон
                                        payload: pub_packet.payload,
                                    });
                                    if let Ok(len) =
                                        mqttrs::encode_slice(&response_packet, &mut out_buf)
                                    {
                                        if let Ok(_) = stream.write_all(&out_buf[..len]) {
                                            let _ = stream.flush();
                                            info!("Oтправлен статус-ответ {}!", payload);
                                        }
                                    }
                                }
                                mqttrs::QosPid::ExactlyOnce(pid) => {
                                    info!("Запрошен QoS 2 (ExactlyOnce) для пакета ID: {:?}", pid);
                                }
                                mqttrs::QosPid::AtMostOnce => {
                                    info!("Сообщение доставлено с QoS 0 (AtMostOnce)");
                                }
                            }
                        }
                        Ok(Some(Packet::Subscribe(sub_packet))) => {
                            info!(
                                "Клиент запросил подписку на топики: {:?}",
                                sub_packet.topics
                            );
                            let mut return_codes = Vec::new();
                            for _ in &sub_packet.topics {
                                return_codes.push(mqttrs::SubscribeReturnCodes::Success(
                                    mqttrs::QoS::AtLeastOnce,
                                ));
                            }
                            let suback = Packet::Suback(mqttrs::Suback {
                                pid: sub_packet.pid,
                                return_codes,
                            });

                            let mut out_buf = [0u8; 128];
                            if let Ok(len) = mqttrs::encode_slice(&suback, &mut out_buf) {
                                stream.write_all(&out_buf[..len])?;
                                stream.flush()?;
                                info!("Отправлен SUBACK для подписок клиента.");
                            }
                        }
                        Ok(Some(Packet::Pingreq)) => {
                            // Ответ на Keep-Alive проверку от клиента (PINGRESP)
                            let pingresp = Packet::Pingresp;
                            let mut out_buf = [0u8; 16];
                            if let Ok(len) = mqttrs::encode_slice(&pingresp, &mut out_buf) {
                                stream.write_all(&out_buf[..len])?;
                                stream.flush()?;
                            }
                        }
                        Ok(Some(other)) => {
                            info!("Получен пакет типа: {:?}", other);
                        }
                        Ok(None) => {
                            // Пакет пришел не полностью, ждем оставшиеся байты в следующем цикле read
                            continue;
                        }
                        Err(e) => {
                            log::error!("Ошибка протокола MQTT: {:?}", e);
                            break;
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    log::error!("Ошибка сокета клиента {:?}: {:?}", client_addr, e);
                    break;
                }
            }
        }
        info!(
            "Поток обработки клиента {:?} завершен. Сокет закрывается.",
            client_addr
        );
        Ok(())
    }
}

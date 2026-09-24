pub mod display;
pub mod mqtt;
pub mod wifi;
pub mod yandex_weather;

use mqttrs::{Connack, ConnectReturnCode, Packet, Publish, QoS};
use slint::Model;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
use ui::MainWindow;

use crate::ui::LightControllEntity;
pub mod ui {
    slint::include_modules!();
}
pub struct Handler {
    map: HashMap<&'static str, Box<dyn Fn(slint::Weak<MainWindow>, String)>>,
}
impl Handler {
    pub fn new() -> Self {
        let mut mp = HashMap::<&str, Box<dyn Fn(slint::Weak<MainWindow>, String)>>::new();
        //row_1
        mp.insert(
            "smart_home/light_controll/kitchen",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g1();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights.iter_mut().find(|entity| entity.name == "kitchen")
                    {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет на кухне включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет на кухне выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g1(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        mp.insert(
            "smart_home/light_controll/hallway",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g1();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights.iter_mut().find(|entity| entity.name == "hallway")
                    {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в прихожей включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в прихожей выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g1(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        mp.insert(
            "smart_home/light_controll/hall",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g1();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights.iter_mut().find(|entity| entity.name == "hall") {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в зале включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в зале выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g1(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        mp.insert(
            "smart_home/light_controll/pantry",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g1();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights.iter_mut().find(|entity| entity.name == "pantry") {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в кладовой включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в кладовой выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g1(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        //row_2
        mp.insert(
            "smart_home/light_controll/controll_center",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g2();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights
                        .iter_mut()
                        .find(|entity| entity.name == "controll_center")
                    {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в центре управления включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в центре управления выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g2(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        mp.insert(
            "smart_home/light_controll/restroom",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g2();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights.iter_mut().find(|entity| entity.name == "restroom")
                    {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в уборной включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в уборной выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g2(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        mp.insert(
            "smart_home/light_controll/bathroom",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g2();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) = lights.iter_mut().find(|entity| entity.name == "bathroom")
                    {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в ванной включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в ванной выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g2(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        mp.insert(
            "smart_home/light_controll/breakroom",
            Box::new(|ui, payload| {
                let ui_clone = ui.clone();
                if let Some(ui) = ui_clone.upgrade() {
                    let current_model = ui.get_light_array_g2();
                    let mut lights: Vec<LightControllEntity> = current_model.iter().collect();
                    if let Some(entity) =
                        lights.iter_mut().find(|entity| entity.name == "breakroom")
                    {
                        if payload == "1" || payload == "on" {
                            entity.is_on = true;
                            println!("Свет в комнате отдыха включен");
                        } else if payload == "0" || payload == "off" {
                            entity.is_on = false;
                            println!("Свет в комнате отдыха выключен");
                        } else {
                            println!("Неизвестное значение {}", payload);
                        }
                    }
                    let new_model = slint::VecModel::from(lights);
                    ui.set_light_array_g2(slint::ModelRc::from(std::rc::Rc::new(new_model)));
                }
            }),
        );
        Self { map: mp }
    }
    pub fn handle_client(
        &self,
        mut stream: TcpStream,
        ui_weak: slint::Weak<ui::MainWindow>,
    ) -> std::io::Result<()> {
        let client_addr = stream.peer_addr()?;
        log::info!("Новый клиент подключился к ESP32: {:?}", client_addr);
        let mut buffer = Box::new([0xffu8; 1024]);
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        loop {
            buffer.fill(0xff);
            match stream.read(&mut *buffer) {
                Ok(0) => {
                    println!("Клиент {:?} закрыл соединение.", client_addr);
                    break;
                }
                Ok(bytes_read) => {
                    let mut data_slice = &buffer[..bytes_read];
                    match mqttrs::decode_slice(&mut data_slice) {
                        Ok(Some(Packet::Connect(connect_packet))) => {
                            println!("MQTT CONNECT от: {}", connect_packet.client_id);

                            // Формируем ответ подтверждения (CONNACK)
                            let connack = Packet::Connack(Connack {
                                session_present: false,
                                code: ConnectReturnCode::Accepted,
                            });

                            // Кодируем пакет в байты
                            let mut out_buf = [0u8; 128];
                            if let Ok(len) = mqttrs::encode_slice(&connack, &mut out_buf) {
                                stream.write_all(&out_buf[..len])?;
                                stream.flush()?;
                                println!("Клиент успешно авторизован!");
                            }
                        }
                        Ok(Some(Packet::Publish(pub_packet))) => {
                            let payload = String::from_utf8_lossy(&pub_packet.payload)
                                .to_string()
                                .to_lowercase();
                            println!("Получен топик [{}]: {}", pub_packet.topic_name, payload);
                            if self.map.contains_key(pub_packet.topic_name) {
                                let f = self.map.get(pub_packet.topic_name).unwrap();
                                f(ui_weak.clone(), payload.clone());
                            } else {
                                println!("Unknown topic: {}", pub_packet.topic_name);
                            }
                            // В mqttrs QoS и Packet ID объединены в поле qospid
                            match pub_packet.qospid {
                                mqttrs::QosPid::AtLeastOnce(pid) => {
                                    let puback = Packet::Puback(pid);
                                    let mut out_buf = [0u8; 64];
                                    if let Ok(len) = mqttrs::encode_slice(&puback, &mut out_buf) {
                                        stream.write_all(&out_buf[..len])?;
                                        stream.flush()?;
                                        println!("Отправлен PUBACK для пакета ID: {:?}", pid);
                                    }
                                    if let Ok(len) = mqttrs::encode_slice(
                                        &Packet::Publish(pub_packet),
                                        &mut out_buf,
                                    ) {
                                        if let Ok(_) = stream.write_all(&out_buf[..len]) {
                                            let _ = stream.flush();
                                            println!(
                                                "Дашборду отправлен статус-ответ {}!",
                                                payload
                                            );
                                        }
                                    }
                                }
                                mqttrs::QosPid::ExactlyOnce(pid) => {
                                    println!(
                                        "Запрошен QoS 2 (ExactlyOnce) для пакета ID: {:?}",
                                        pid
                                    );
                                }
                                mqttrs::QosPid::AtMostOnce => {
                                    println!("Сообщение доставлено с QoS 0 (AtMostOnce)");
                                }
                            }

                            // let mut send_buf = [0u8; 128];
                            // if let Ok(len) =
                            //     mqttrs::encode_slice(&Packet::Publish(pub_packet), &mut send_buf)
                            // {
                            //     if let Ok(_) = stream.write_all(&send_buf[..len]) {
                            //         let _ = stream.flush();
                            //         println!("Дашборду отправлен статус-ответ ON!");
                            //     }
                            // }
                        }
                        Ok(Some(Packet::Subscribe(sub_packet))) => {
                            println!(
                                "Клиент запросил подписку на топики: {:?}",
                                sub_packet.topics
                            );
                            // Формируем коды возврата для каждого топика
                            let mut return_codes = Vec::new();
                            for _ in &sub_packet.topics {
                                // ИСПРАВЛЕНО: передаем успешный статус с уровнем QoS 0 (AtMostOnce)
                                return_codes
                                    .push(mqttrs::SubscribeReturnCodes::Success(QoS::AtLeastOnce));
                            }
                            // Создаем пакет SUBACK
                            let suback = Packet::Suback(mqttrs::Suback {
                                pid: sub_packet.pid,
                                return_codes,
                            });

                            let mut out_buf = [0u8; 128];
                            if let Ok(len) = mqttrs::encode_slice(&suback, &mut out_buf) {
                                stream.write_all(&out_buf[..len])?;
                                stream.flush()?;
                                println!("Отправлен SUBACK для подписок клиента.");
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
                            println!("Получен пакет типа: {:?}", other);
                        }
                        Ok(None) => {
                            // Пакет пришел не полностью, ждем оставшиеся байты в следующем цикле read
                            continue;
                        }
                        Err(e) => {
                            println!("Ошибка протокола MQTT: {:?}", e);
                            break;
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    println!("Ошибка сокета клиента {:?}: {:?}", client_addr, e);
                    break;
                }
            }
        }
        Ok(())
    }
}

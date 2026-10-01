pub mod display;
pub mod wifi;
pub mod yandex_weather;
use bmp180_driver::BMP180;
use chrono::Local;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::{info, warn};
use mqttrs::{Connack, ConnectReturnCode, Packet, Publish, QoS};
use slint::{ComponentHandle, ToSharedString, Weak};
use slint::{Model, SharedString, Timer};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use crate::yandex_weather::YandexWeatherForeCasts;
use crate::{ui::LightControllEntity, yandex_weather::YandexWeatherFact};
pub mod ui {
    slint::include_modules!();
}
use ui::MainWindow;
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

pub fn timer_block(ui: Weak<MainWindow>) -> Timer {
    let timer_fn = move || {
        let time = Local::now();
        let weak = YandexWeatherFact::get_rus_weekday(&time).to_shared_string();
        let time = time.format("%H:%M %d.%m.%Y").to_shared_string();
        if let Some(ui) = ui.upgrade() {
            ui.set_global_time(format!("{}, {}", weak, time).into());
        }
    };
    let timer = Timer::default();
    timer.start(slint::TimerMode::Repeated, Duration::from_secs(1), timer_fn);
    timer
}
pub fn bmp_block(ui: Weak<MainWindow>, i2c_driver: Rc<RefCell<I2cDriver<'static>>>) -> Timer {
    let timer_bmp = Timer::default();
    let i2c_driver_clone = i2c_driver.clone();
    let mut bmp_holder = None;

    let mut timer_bmp_action = move || {
        let mut i2c_ref = i2c_driver_clone.borrow_mut();
        if bmp_holder.is_none() {
            unsafe {
                let static_i2c: &mut I2cDriver<'static> = std::mem::transmute(&mut *i2c_ref);
                let bmp_init = BMP180::new(static_i2c, esp_idf_hal::delay::FreeRtos);

                if let Ok(bmp_ready) = bmp_init.initialize() {
                    bmp_holder = Some(bmp_ready);
                }
            }
        }
        if let Some(ref mut bmp180) = bmp_holder {
            let (t, p, a) = bmp180
                .read_all(bmp180_driver::Resolution::Standard)
                .unwrap();
            if let Some(ui) = ui.upgrade() {
                ui.set_global_temp(format!("{:.1}", t).into());
                let pressure_mm_hg = p as f64 / 133.3224;
                let press_str = format!("{:.0} ", pressure_mm_hg);
                ui.set_global_pressure(press_str.into());
                ui.set_global_altitude(format!("{:.0}", a).into());
            }
        }
    };
    timer_bmp_action();
    timer_bmp.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(10),
        timer_bmp_action,
    );
    timer_bmp
}
pub fn yandex_weather_block(ui: Weak<MainWindow>) -> Timer {
    let yandex_weather_thread = move || {
        if let Some(ui) = ui.upgrade() {
            const YANDEX_API_KEY: &str = "103c2085-4f31-4bf1-9e41-672027e29704";
            const LAT: &str = "53.2355"; // Координаты Клинцов
            const LON: &str = "34.3539";
            let config = Configuration {
                timeout: Some(Duration::from_secs(10)),
                buffer_size: Some(1024 * 8),
                buffer_size_tx: Some(512 * 2),
                ..Default::default()
            };
            match EspHttpConnection::new(&config) {
                Ok(mut connection) => {
                    println!("Запрос уличной погоды к Яндекс API...");
                    match YandexWeatherFact::connection(&mut connection, YANDEX_API_KEY, LAT, LON) {
                        Ok(result) => {
                            match YandexWeatherFact::parse(&result) {
                                Ok(w) => {
                                    ui.set_global_weather_status("online".into());
                                    ui.set_global_weather(ui::WeatherFact {
                                        status: w.condition.clone(),
                                        cloudness: w.cloudness,
                                        condition: YandexWeatherFact::translate_condition(
                                            &w.condition,
                                        ),
                                        daytime: w.daytime,
                                        feels_like: w.feels_like,
                                        humidity: w.humidity,
                                        icon: w.icon.into(),
                                        is_thunder: w.is_thunder,
                                        prec_prob: w.prec_prob,
                                        prec_type: w.prec_type,
                                        pressure_mm: w.pressure_mm,
                                        pressure_pa: w.pressure_pa,
                                        season: w.season.into(),
                                        temp: w.temp,
                                        wind_dir: YandexWeatherFact::translate_wind_dir(
                                            &w.wind_dir,
                                        ),
                                        wind_gust: w.wind_gust.into(),
                                        wind_speed: w.wind_speed.into(),
                                    });
                                }
                                Err(e) => {
                                    println!("Ошибка получения погоды: {}", e);
                                }
                            }
                            match YandexWeatherForeCasts::parse(&result) {
                                Ok(v) => {
                                    let (today, _tomorrow) = v;
                                    println!("{:#?}", today);
                                }
                                Err(e) => {
                                    println!("Ошибка получения погоды: {}", e);
                                }
                            }
                        }
                        Err(e) => println!("{}", e),
                    }
                }
                Err(err) => println!("Не могу создать соединение ... {}", err),
            }
        }
    };
    let weather_timer = Timer::default();
    yandex_weather_thread();
    weather_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(45 * 60),
        yandex_weather_thread,
    );
    weather_timer
}

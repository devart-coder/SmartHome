pub mod display;
pub mod security;
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
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use crate::ui::LightObject;
use crate::yandex_weather::YandexWeatherForeCasts;
pub mod ui {
    slint::include_modules!();
}
use crate::yandex_weather::YandexWeatherFact;
use ui::MainWindow;
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
                                    let (today, tomorrow) = v;
                                    // println!("{:#?}", today);
                                    // println!("{:#?}", tomorrow);
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

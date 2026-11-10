pub mod display;
pub mod mqtt_manager;
pub mod security;
pub mod wifi;
pub mod yandex_weather;

use crate::ui::GlobalWeather;
use crate::yandex_weather::YandexWeatherForeCasts;
use bmp180_driver::BMP180;
use chrono::Local;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use log::error;
use slint::{ComponentHandle, Global, Timer};
use slint::{ToSharedString, Weak};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
pub mod ui {
    slint::include_modules!();
}
use crate::yandex_weather::YandexWeatherFact;
use ui::GlobalBMP180;
use ui::GlobalConnectionsStatus;
use ui::GlobalMainTab;
use ui::MainWindow;
#[macro_export]
macro_rules! to_lcd {
    ($msg:expr,$display:ident,$ui:ident) => {
        info!("{}", $msg);
        let mut local_str = $ui.get_boot_logs();
        local_str.push_str($msg);
        $ui.set_boot_logs(local_str);
        $display.update();
    };
}
#[macro_export]
macro_rules! to_lcdln {
    ($msg:expr,$display:ident,$ui:ident) => {
        let e = $msg.to_string() + "\n";
        to_lcd!(e.as_str(), $display, $ui)
    };
}
pub fn timer_block(ui: Weak<MainWindow>) -> Timer {
    let timer_fn = move || {
        let time = Local::now();
        let weak = YandexWeatherFact::get_rus_weekday(&time).to_shared_string();
        let time = time.format("%H:%M %d.%m.%Y").to_shared_string();
        match ui.upgrade() {
            Some(ui) => {
                ui.global::<GlobalMainTab>()
                    .set_time(format!("{}, {}", weak, time).into());
            }
            None => error!("[Error]: time_block"),
        }
    };
    let timer = Timer::default();
    timer.start(slint::TimerMode::Repeated, Duration::from_secs(1), timer_fn);
    timer
}
pub fn bmp_block(ui: Weak<MainWindow>, i2c_driver: Rc<RefCell<I2cDriver<'static>>>) -> Timer {
    let timer_bmp = Timer::default();
    let mut bmp_holder = None;

    let mut timer_bmp_action = move || {
        let mut i2c_ref = i2c_driver.borrow_mut();
        if bmp_holder.is_none() {
            unsafe {
                let static_i2c: &mut I2cDriver<'static> = std::mem::transmute(&mut *i2c_ref);
                let bmp_init = BMP180::new(static_i2c, FreeRtos);
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
                let bmp_ui = ui.global::<GlobalBMP180>();
                bmp_ui.set_temperature(format!("{:.1}", t).into());
                let pressure_mm_hg = p as f64 / 133.3224;
                let press_str = format!("{:.0} ", pressure_mm_hg);
                bmp_ui.set_pressure(press_str.into());
                bmp_ui.set_altitude(format!("{:.0}", a).into());
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
                                    let global_weather = ui.global::<GlobalWeather>();
                                    global_weather.set_weather_status("online".into());
                                    global_weather.set_weather(ui::WeatherFact {
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
                                    let (_today, _tomorrow) = v;
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

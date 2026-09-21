use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use bmp180_driver::BMP180;
use chrono::Local;
use esp_idf_hal::delay::{Delay, FreeRtos};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::AuthMethod;
use lcd_touch_display::display::Display;
use lcd_touch_display::wifi::Wifi;
use lcd_touch_display::yandex_weather::YandexWeatherFact;
use slint::platform::update_timers_and_animations;
use slint::{
    ComponentHandle, SharedString,
};

use crate::ui::{MainWindow, WeatherFact};
mod ui {
    slint::include_modules!();
}
fn main() {
    esp_idf_sys::link_patches();

    //------------------------------------------------------------------------//
    //DisplayWindowInit
    let peripherals = Peripherals::take().unwrap();
    let p = peripherals;
    let i2c_driver = Rc::new(RefCell::new(
        I2cDriver::new(
            p.i2c0,
            p.pins.gpio8, // SDA
            p.pins.gpio9, // SCL
            &I2cConfig::new().baudrate(Hertz(400_000)),
        )
        .unwrap(),
    ));
    let display = Display::default(i2c_driver.clone());

    //------------------------------------------------------------------------//
    //SlintMainWindowDraw
    let main_window = MainWindow::new().unwrap();
    main_window.show().unwrap();
    display.update();
    FreeRtos::delay_ms(100);
    macro_rules! log_to_lcd {
        ($msg:expr) => {
            println!("{}", $msg);
            let mut local_str = main_window.get_boot_logs();
            local_str.push_str($msg);
            local_str.push_str("\n");
            main_window.set_boot_logs(local_str);
            update_timers_and_animations();
            display.update();
            FreeRtos::delay_ms(100);
        };
    }
    //-------------------------------------------------//
    //---------------------_WIFI_----------------------//
    log_to_lcd!("Инициализация Wi-Fi...");
    let mut wifi = Wifi::new(p.modem);
    wifi.init_config(|b| {
        b.set_ssid("TP-Link_D88D".to_string())
            .set_password("14236679".to_string())
            .set_auth_method(AuthMethod::WPA2Personal)
    });
    wifi.show_connection(|w| {
        log_to_lcd!("Подключение к Wi-Fi...");
        w.start().unwrap();

        log_to_lcd!("Поиск сети...");
        w.connect().unwrap();

        log_to_lcd!("Сеть найдена! Получение IP-адреса (DHCP)...");
        w.wait_netif_up().unwrap();

        log_to_lcd!("Wi-Fi успешно подключен!");
        main_window.set_global_wifi_status("online".into());
    });
    //-------------------------------------------------//
    //--------------------_LocalData_------------------//
    log_to_lcd!("Инициализация времени...");
    let sntp = EspSntp::new_default().unwrap();
    log_to_lcd!("Ожидание точного времени...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        FreeRtos::delay_ms(500);
    }
    unsafe {
        std::env::set_var("TZ", "MSK-3");
        esp_idf_sys::tzset();
    }
    log_to_lcd!("Время успешно синхронизировано!");
    main_window.set_is_loading(false);
    //-------------------------------------------------//
    //---------------------_TIMER_DATA_----------------//
    let timer = slint::Timer::default();
    {
        let ui_time = main_window.as_weak();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                let time = Local::now();
                let weak = SharedString::from(YandexWeatherFact::get_rus_weekday(&time));
                let time = time.format("%H:%M %d.%m.%Y").to_string();
                if let Some(ui) = ui_time.upgrade() {
                    ui.set_global_time(format!("{}, {}", weak, time).into());
                }
            },
        );
    }
    //-------------------------------------------------//
    //---------------------_TIMER_BMP_-----------------//
    let timer_bmp = slint::Timer::default();
    {
        let ui_bmp = main_window.as_weak();
        let timer_bmp_action = move || {
            let mut i2c_ref = i2c_driver.borrow_mut();
            let bmp180 = BMP180::new(&mut *i2c_ref, Delay::default());
            let mut bmp180 = bmp180.initialize().expect("BMP not init");
            let (t, p, a) = bmp180
                .read_all(bmp180_driver::Resolution::Standard)
                .unwrap();
            if let Some(ui) = ui_bmp.upgrade() {
                ui.set_global_temp(format!("{:.1}", t).into());
                let pressure_mm_hg = p as f64 / 133.3224;
                let press_str = format!("{:.1} ", pressure_mm_hg);
                ui.set_global_pressure(press_str.into());
                ui.set_global_altitude(format!("{:.0}", a).into());
            }
        };
        timer_bmp.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(10),
            timer_bmp_action,
        );
    }
    //-------------------------------------------------//
    //------------------_YandexWeather_----------------//
    let weather_timer = slint::Timer::default();
    {
        let ui_weather_weak = main_window.as_weak();
        let yandex_weather_thread = move || {
            if let Some(ui) = ui_weather_weak.upgrade() {
                const YANDEX_API_KEY: &str = "103c2085-4f31-4bf1-9e41-672027e29704";
                const LAT: &str = "53.2355"; // Координаты Клинцов
                const LON: &str = "34.3539";
                let config = Configuration {
                    timeout: Some(Duration::from_secs(10)),
                    buffer_size: Some(1024),
                    buffer_size_tx: Some(512),
                    ..Default::default()
                };
                match EspHttpConnection::new(&config) {
                    Ok(mut connection) => {
                        println!("Запрос уличной погоды к Яндекс API...");
                        match YandexWeatherFact::connection(
                            &mut connection,
                            YANDEX_API_KEY,
                            LAT,
                            LON,
                        ) {
                            Ok(result) => match YandexWeatherFact::parse(result) {
                                Ok(w) => {
                                    ui.set_global_weather_status("online".into());
                                    ui.set_global_weather(WeatherFact {
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
                            },
                            Err(e) => println!("{}", e),
                        }
                    }
                    Err(err) => println!("Не могу создать соединение ... {}", err),
                }
            }
        };
        yandex_weather_thread();
        weather_timer.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(45 * 60),
            yandex_weather_thread,
        );
    }
    //-------------------------------------------------//
    display.touch_pad_loop();
}

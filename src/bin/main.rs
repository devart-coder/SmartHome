use esp_idf_hal::delay::{Delay, FreeRtos};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::http::client::{Configuration, Connection, EspHttpConnection};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::ota::EspOta;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::sys::{ESP_ERR_IMAGE_INVALID, ESP_ERR_INVALID_RESPONSE, EspError};
use esp_idf_svc::wifi::AuthMethod;
use log::{error, info, warn};
use master_display::display::Display;
use master_display::ui::{MainWindow, WeatherFact};
use master_display::wifi::Wifi;
use master_display::{Handler, bmp_block, timer_block, yandex_weather_block};
use slint::platform::update_timers_and_animations;
use slint::{ComponentHandle, SharedString, Timer};
use std::cell::RefCell;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
fn main() {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);
    //------------------------------------------------------------------------//
    //DisplayWindowInit
    let p = Peripherals::take().expect("Владение перифирией было захвачено где-то ранее.");
    let i2c_driver = Rc::new(RefCell::new(
        I2cDriver::new(
            p.i2c0,
            p.pins.gpio8, // SDA
            p.pins.gpio9, // SCL
            &I2cConfig::new().baudrate(Hertz(400_000)),
        )
        .expect("I2C ошибка захвата владения."),
    ));
    let mut display = Display::default(i2c_driver.clone());
    //------------------------------------------------------------------------//
    //SlintMainWindowDraw
    let main_window = MainWindow::new().unwrap();
    match main_window.show() {
        Ok(_) => info!("ui:: выполнен метод show"),
        Err(_) => warn!("ui:: методд не позможно выполнить"),
    };
    main_window.set_is_loading(true);
    FreeRtos::delay_ms(100);
    macro_rules! log_to_lcdln {
        ($msg:expr) => {
            info!("{}", $msg);
            let mut local_str = main_window.get_boot_logs();
            local_str.push_str($msg);
            local_str.push_str("\n");
            main_window.set_boot_logs(local_str);
            update_timers_and_animations();
            display.update();
            FreeRtos::delay_ms(100);
        };
    }
    macro_rules! log_to_lcd {
        ($msg:expr) => {
            info!("{}", $msg);
            let mut local_str = main_window.get_boot_logs();
            local_str.push_str($msg);
            main_window.set_boot_logs(local_str);
            update_timers_and_animations();
            display.update();
            FreeRtos::delay_ms(100);
        };
    }
    //-------------------------------------------------//
    //---------------------_WIFI_----------------------//
    log_to_lcdln!("Инициализация Wi-Fi...");
    let mut wifi = Wifi::new(p.modem);
    wifi.init_config(|b| {
        b.set_ssid("TP-Link_D88D")
            .set_password("14236679")
            .set_auth_method(AuthMethod::WPA2Personal)
    });
    wifi.show_connection(|w| {
        log_to_lcd!("Подключение к Wi-Fi... ");
        match w.start() {
            Ok(_) => {
                log_to_lcdln!("✅");
            }
            Err(err) => {
                log_to_lcdln!(format!("Error {}", err).as_str());
            }
        };

        log_to_lcd!("Поиск сети... ");
        match w.connect() {
            Ok(_) => {
                log_to_lcdln!("✅");
                log_to_lcd!("Получение IP-адреса (DHCP)...");
                match w.wait_netif_up() {
                    Ok(_) => {
                        if let Ok(ip_info) = w.wifi().sta_netif().get_ip_info() {
                            log_to_lcdln!(format!("✅\nIp: {:?}", ip_info.ip).as_str());
                        } else {
                            log_to_lcdln!("✅ (No IP)");
                        }
                    }
                    Err(err) => {
                        log_to_lcdln!(format!("Error {}", err).as_str());
                    }
                };
            }
            Err(err) => {
                log_to_lcdln!(format!("Error {}", err).as_str());
            }
        };
        log_to_lcdln!("Wi-Fi успешно подключен!");
        unsafe {
            esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
            println!("Режим энергосбережения Wi-Fi успешно ОТКЛЮЧЕН.");
        }
    });
    main_window.set_global_wifi_status("online".into());
    //-------------------------------------------------//
    //--------------------_LocalData_------------------//
    log_to_lcdln!("Инициализация времени...");
    let sntp = EspSntp::new_default().unwrap();
    log_to_lcdln!("Ожидание точного времени...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        log_to_lcd!("{} ");
        FreeRtos::delay_ms(1000);
    }
    log_to_lcdln!("");
    unsafe {
        std::env::set_var("TZ", "MSK-3");
        esp_idf_sys::tzset();
    }
    log_to_lcdln!("Время успешно синхронизировано!");
    main_window.set_is_loading(false);
    //-------------------------------------------------//
    let _timer = timer_block(main_window.as_weak());
    let _timer_bmp = bmp_block(main_window.as_weak(), i2c_driver);
    let _yandex_timer = yandex_weather_block(main_window.as_weak());
    let ui_weak = main_window.as_weak();
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:1883").unwrap();
        info!("MQTT Сервер успешно запущен на ESP32 и слушает порт 1883...");
        for stream in listener.incoming() {
            match stream {
                Ok(socket) => {
                    let ui_for_client = ui_weak.clone();
                    thread::spawn(move || {
                        let h = Handler::new();
                        if let Err(e) = h.handle_client(socket, ui_for_client.clone()) {
                            error!("Сессия завершена с ошибкой: {:?}", e);
                        }
                    });
                }
                Err(e) => error!("TCP Error: {:?}", e),
            }
        }
    });
    //---------------------------------------------------------//
    loop {
        display.handle_touch_pad();
    }
}

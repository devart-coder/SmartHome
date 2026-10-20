use esp_idf_hal::delay::{Delay, FreeRtos};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::mqtt::client::{
    EspAsyncMqttClient, EspMqttClient, EspMqttConnection, EventPayload, MqttClientConfiguration,
    QoS,
};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::AuthMethod;
use log::{error, info, warn};
use master_display::display::Display;
use master_display::security::{BROKER_URL, LightSubscriptions, PASSWORD, SDDI};
use master_display::ui::{LightObject, MainWindow, WeatherFact};
use master_display::wifi::Wifi;
use master_display::{bmp_block, timer_block, yandex_weather_block};
use slint::platform::update_timers_and_animations;
use slint::{
    ComponentHandle, Model, ModelExt, SharedString, Timer, VecModel, invoke_from_event_loop,
};
use std::cell::RefCell;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread::{self, spawn};
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
    //-------------------------------------------------//
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
        b.set_ssid(SDDI)
            .set_password(PASSWORD)
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
    let mut counter = 1;
    while sntp.get_sync_status() != SyncStatus::Completed {
        log_to_lcd!(format!("{} ", counter).as_str());
        FreeRtos::delay_ms(1000);
        if counter <= 10 {
            counter += 1;
        } else {
            log_to_lcdln!("times fail...");
            break;
        }
    }
    if counter <= 10 {
        unsafe {
            std::env::set_var("TZ", "MSK-3");
            esp_idf_sys::tzset();
        }
        log_to_lcdln!("\nВремя успешно синхронизировано!");
    }
    main_window.set_is_loading(false);
    display.update();
    //-------------------------------------------------//
    let _timer = timer_block(main_window.as_weak());
    let _timer_bmp = bmp_block(main_window.as_weak(), i2c_driver);
    //-------------------------------------------------//
    let (client, mut connection) = EspMqttClient::new(
        BROKER_URL,
        &MqttClientConfiguration {
            client_id: Some("Display_7'"),
            keep_alive_interval: Some(Duration::from_secs(60)),
            ..Default::default()
        },
    )
    .unwrap();
    let client_guard = Arc::new(Mutex::new(client));
    let c = client_guard.clone();
    std::thread::spawn(move || {
        info!("Фоновый поток MQTT запущен, ожидание событий...");
        while let Ok(event) = connection.next() {
            match event.payload() {
                EventPayload::Connected(_) => {
                    info!("Успешно подключились к MQTT-брокеру!");
                    let c = c.clone();
                    let subscribe = move || {
                        let mut cc = c.lock().unwrap();
                        match cc.subscribe(
                            LightSubscriptions::ControllRoom.path().as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/controll_center"
                                );
                            }
                        }
                        match cc.subscribe(
                            (LightSubscriptions::ControllRoom.path() + "/light_ownership").as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/controll_center/ownership"
                                );
                            }
                        }
                        match cc.subscribe(
                            LightSubscriptions::Restroom.path().as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/restroom"
                                );
                            }
                        }
                        match cc.subscribe(
                            (LightSubscriptions::Restroom.path() + "/light_ownership").as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/restroom"
                                );
                            }
                        }
                        match cc.subscribe(
                            LightSubscriptions::Bathroom.path().as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/bathroom"
                                );
                            }
                        }
                        match cc.subscribe(
                            (LightSubscriptions::Bathroom.path() + "/light_ownership").as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/bathroom"
                                );
                            }
                        }
                        match cc.subscribe(
                            LightSubscriptions::Breakroom.path().as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/breakroom"
                                );
                            }
                        }
                        match cc.subscribe(
                            (LightSubscriptions::Breakroom.path() + "/light_ownership").as_str(),
                            QoS::AtLeastOnce,
                        ) {
                            Err(e) => error!("Не удалось подписаться на топик: {:?}", e),
                            Ok(_) => {
                                info!(
                                    "Успешная подписка на топик: smart_home/light_controll/breakroom"
                                );
                            }
                        }
                    };
                    thread::spawn(subscribe);
                }
                EventPayload::Disconnected => {
                    warn!("Потеряно соединение с MQTT-брокером. Переподключение...");
                }
                EventPayload::Error(e) => {
                    log::error!("Error: {}", e.to_string())
                }
                _ => {}
            }
        }
    });
    let c = client_guard.clone();
    main_window.on_g2_status(move |name, status| {
        info!("Name : {} Status:{}", name, status);
        if name == LightSubscriptions::ControllRoom.name() {
            if status == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::ControllRoom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::ControllRoom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        } else if name == LightSubscriptions::Restroom.name() {
            if status == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Restroom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Restroom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        } else if name == LightSubscriptions::Bathroom.name() {
            if status == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Bathroom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Bathroom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        } else if name == LightSubscriptions::Breakroom.name() {
            if status == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Breakroom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Breakroom.path().as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        }
    });
    let c = client_guard.clone();
    main_window.on_g2_ownership(move |name, ownership| {
        info!("Name : {} Ownership:{}", name, ownership);
        if name == LightSubscriptions::ControllRoom.name() {
            if ownership == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::ControllRoom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::ControllRoom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        } else if name == LightSubscriptions::Restroom.name() {
            if ownership == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Restroom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Restroom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        } else if name == LightSubscriptions::Bathroom.name() {
            if ownership == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Bathroom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Bathroom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        } else if name == LightSubscriptions::Breakroom.name() {
            if ownership == true {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Breakroom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"on",
                    )
                    .unwrap();
            } else {
                c.lock()
                    .unwrap()
                    .publish(
                        LightSubscriptions::Breakroom
                            .with_ownership_suffix()
                            .as_str(),
                        QoS::AtLeastOnce,
                        true,
                        b"off",
                    )
                    .unwrap();
            }
        }
    });
    //---------------------------------------------------------//
    loop {
        display.handle_touch_pad();
    }
}

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::mqtt::client::MqttClientConfiguration;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::AuthMethod;
use log::{error, info};
use master_display::display::Display;
use master_display::mqtt_manager::MqttClientManager;
use master_display::security::{BROKER_URL, PASSWORD, SDDI};
use master_display::ui::{GlobalConnectionsStatus, MainWindow};
use master_display::wifi::Wifi;
use master_display::{bmp_block, timer_block, to_lcd, to_lcdln};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
fn main() {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);
    //------------------------------------------------------------------------//
    //DisplayWindowInit
    let p = Peripherals::take().expect("Владение перифирией было захвачено где-то ранее.");
    Rc::new(RefCell::new(&p));
    let i2c_driver = I2cDriver::new(
        p.i2c0,
        p.pins.gpio8, // SDA
        p.pins.gpio9, // SCL
        &I2cConfig::new().baudrate(100_000.into()),
    )
    .unwrap();
    let i2c_driver = Rc::new(RefCell::new(i2c_driver));
    let mut display = Display::default(i2c_driver.clone());
    //------------------------------------------------------------------------//
    //SlintMainWindow
    let main_window = MainWindow::new().unwrap();
    main_window.set_is_loading(true);
    //-------------------------------------------------//
    //---------------------_WIFI_----------------------//
    to_lcdln!("Инициализация Wi-Fi...", display, main_window);
    let mut wifi = Wifi::new(p.modem);
    wifi.init_config(|b| {
        b.set_ssid(SDDI)
            .set_password(PASSWORD)
            .set_auth_method(AuthMethod::WPA2Personal)
    });
    wifi.show_connection(|w| {
        to_lcd!("Подключение к Wi-Fi... ", display, main_window);
        match w.start() {
            Ok(_) => {
                to_lcdln!("✅", display, main_window);
            }
            Err(err) => {
                to_lcdln!(format!("Error {}", err).as_str(), display, main_window);
            }
        };
        to_lcd!("Поиск сети... ", display, main_window);
        match w.connect() {
            Ok(_) => {
                to_lcdln!("✅", display, main_window);
                to_lcd!("Получение IP-адреса (DHCP)...", display, main_window);
                match w.wait_netif_up() {
                    Ok(_) => {
                        if let Ok(ip_info) = w.wifi().sta_netif().get_ip_info() {
                            to_lcdln!(
                                format!("✅\nIp: {:?}", ip_info.ip).as_str(),
                                display,
                                main_window
                            );
                            main_window
                                .global::<GlobalConnectionsStatus>()
                                .set_wifi_status("online".into());
                            //-------------------------------------------------//
                            //--------------------_LocalData_------------------//
                            to_lcdln!("Инициализация времени...", display, main_window);
                            let thread = thread::spawn(move || {
                                let mut counter = 1;
                                let sntp = EspSntp::new_default().unwrap();
                                while sntp.get_sync_status() != SyncStatus::Completed {
                                    FreeRtos::delay_ms(1000);
                                    if counter <= 10 {
                                        counter += 1;
                                    } else {
                                        break;
                                    }
                                }
                                if counter <= 10 {
                                    unsafe {
                                        std::env::set_var("TZ", "MSK-3");
                                        esp_idf_sys::tzset();
                                    }
                                    return SyncStatus::Completed;
                                }
                                sntp.get_sync_status()
                            });
                            let r = thread.join().unwrap();
                            match r {
                                SyncStatus::Completed => {
                                    to_lcd!("Время успешно синхронизировано", display, main_window);
                                }
                                _ => {
                                    to_lcd!("Ошибка синхронизации времени", display, main_window);
                                }
                            };
                        } else {
                            to_lcdln!("✅ (No IP)", display, main_window);
                        }
                    }
                    Err(err) => {
                        to_lcdln!(format!("Error {}", err).as_str(), display, main_window);
                    }
                };
            }
            Err(err) => {
                to_lcdln!(format!("Error {}", err).as_str(), display, main_window);
            }
        };
        unsafe {
            esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
            println!("Режим энергосбережения Wi-Fi успешно ОТКЛЮЧЕН.");
        }
        to_lcdln!("Wi-Fi успешно подключен!", display, main_window);
    });
    main_window.set_is_loading(false);
    //-------------------------------------------------//
    let _timer = timer_block(main_window.as_weak());
    let _ui_timer_bmp = bmp_block(main_window.as_weak(), i2c_driver.clone());
    display.update();
    //-------------------------------------------------//
    let _ui = main_window.as_weak();
    let (tx, rx) = channel::<&str>();
    if let Err(e) = tx.send("connecting") {
        error!("[SendError]: {}", e);
    }
    let mrx = Arc::new(Mutex::new(rx));
    let _mrx_clone = mrx.clone();
    let mut client = Arc::new(Mutex::new(MqttClientManager::new(
        BROKER_URL,
        MqttClientConfiguration {
            client_id: Some("Display_7"),
            keep_alive_interval: Some(Duration::from_secs(10)),
            disable_clean_session: false,
            username: None,
            password: None,
            ..Default::default()
        },
    )));
    client.lock().unwrap().run();
    // let c = client_guard.clone();
    // main_window
    //     .global::<GlobalLightControllGroups>()
    //     .on_g2_status(move |name, status| {
    //         info!("Name : {} Status:{}", name, status);
    //         let mut c_lock = c.lock().unwrap();
    //         if name == LightSubscriptions::ControllRoom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::ControllRoom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::ControllRoom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         } else if name == LightSubscriptions::Restroom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Restroom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Restroom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         } else if name == LightSubscriptions::Bathroom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Bathroom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Bathroom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         } else if name == LightSubscriptions::Breakroom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Breakroom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Breakroom.topic().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         }
    //     });
    // let c = client_guard.clone();
    // main_window
    //     .global::<GlobalLightControllGroups>()
    //     .on_g2_ownership(move |name, status| {
    //         info!("Name : {} Status:{}", name, status);
    //         let mut c_lock = c.lock().unwrap();
    //         if name == LightSubscriptions::ControllRoom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::ControllRoom
    //                             .topic_with_ownership()
    //                             .as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::ControllRoom
    //                             .topic_with_ownership()
    //                             .as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         } else if name == LightSubscriptions::Restroom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Restroom.topic_with_ownership().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Restroom.topic_with_ownership().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         } else if name == LightSubscriptions::Bathroom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Bathroom.topic_with_ownership().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Bathroom.topic_with_ownership().as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         } else if name == LightSubscriptions::Breakroom.name() {
    //             if status == true {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Breakroom
    //                             .topic_with_ownership()
    //                             .as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"on",
    //                     )
    //                     .unwrap();
    //             } else {
    //                 c_lock
    //                     .publish(
    //                         LightSubscriptions::Breakroom
    //                             .topic_with_ownership()
    //                             .as_str(),
    //                         QoS::AtLeastOnce,
    //                         true,
    //                         b"off",
    //                     )
    //                     .unwrap();
    //             }
    //         }
    //     });
    let ui = main_window.as_weak();

    let client = client.clone();
    main_window
        .global::<GlobalConnectionsStatus>()
        .on_mqtt_button_pressed(move || {
            let current_mqtt_status = ui
                .unwrap()
                .global::<GlobalConnectionsStatus>()
                .get_mqtt_status();
            if current_mqtt_status == "disconnected" {
                ui.unwrap()
                    .global::<GlobalConnectionsStatus>()
                    .set_mqtt_status("connected".into());
                client.lock().unwrap().restart();
                info!("touch connected");
            } else if current_mqtt_status == "connected" {
                ui.unwrap()
                    .global::<GlobalConnectionsStatus>()
                    .set_mqtt_status("disconnected".into());
                info!("touch disconnected");
            }
        });
    //---------------------------------------------------------//
    loop {
        display.handle_touch_pad();
    }
}

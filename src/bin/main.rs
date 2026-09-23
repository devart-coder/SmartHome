use bmp180_driver::BMP180;
use chrono::Local;
use esp_idf_hal::delay::{Delay, FreeRtos};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_svc::ota::EspOta;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::AuthMethod;
use lcd_touch_display::display::Display;
use lcd_touch_display::wifi::Wifi;
use lcd_touch_display::yandex_weather::YandexWeatherFact;
use slint::platform::update_timers_and_animations;
use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::ui::{MainWindow, WeatherFact};
use esp_idf_svc::http::Headers;
use std::io::{Read, Write};
mod ui {
    slint::include_modules!();
}
use mqttrs::{Connack, Connect, ConnectReturnCode, Packet, Pid, QoS};

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    // Выделяем буфер под входящие данные.
    // Для экономии памяти на ESP32 1024 байт под пакет — более чем достаточно.
    let mut buffer = vec![0u8; 1024];
    let client_addr = stream.peer_addr()?;
    log::info!("Новый клиент подключился к ESP32: {:?}", client_addr);

    stream.set_read_timeout(Some(Duration::from_secs(30)))?;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Клиент {:?} закрыл соединение.", client_addr);
                break;
            }
            Ok(bytes_read) => {
                // Берем срез буфера с реально прочитанными байтами
                let mut data_slice = &buffer[..bytes_read];

                // Декодируем пакет с помощью mqttrs::decode_slice
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
                        let payload = String::from_utf8_lossy(&pub_packet.payload);
                        println!("Получен топик [{}]: {}", pub_packet.topic_name, payload);

                        // В mqttrs QoS и Packet ID объединены в поле qospid
                        match pub_packet.qospid {
                            mqttrs::QosPid::AtLeastOnce(pid) => {
                                // Если QoS = AtLeastOnce (QoS 1), мы ОБЯЗАНЫ отправить PUBACK с этим же pid
                                let puback = Packet::Puback(pid);
                                let mut out_buf = [0u8; 64];

                                if let Ok(len) = mqttrs::encode_slice(&puback, &mut out_buf) {
                                    stream.write_all(&out_buf[..len])?;
                                    stream.flush()?;
                                    println!("Отправлен PUBACK для пакета ID: {:?}", pid);
                                }
                            }
                            mqttrs::QosPid::ExactlyOnce(pid) => {
                                println!("Запрошен QoS 2 (ExactlyOnce) для пакета ID: {:?}", pid);
                            }
                            mqttrs::QosPid::AtMostOnce => {
                                println!("Сообщение доставлено с QoS 0 (AtMostOnce)");
                            }
                        }
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
                                .push(mqttrs::SubscribeReturnCodes::Success(QoS::AtMostOnce));
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
    main_window.set_is_loading(true);
    display.update();
    FreeRtos::delay_ms(100);
    macro_rules! log_to_lcdln {
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
    macro_rules! log_to_lcd {
        ($msg:expr) => {
            println!("{}", $msg);
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
        b.set_ssid("TP-Link_D88D".to_string())
            .set_password("14236679".to_string())
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
                            log_to_lcdln!(format!("✅ Ip: {:?}", ip_info.ip).as_str());
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
    });
    unsafe {
        esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
    }
    println!("Режим энергосбережения Wi-Fi успешно ОТКЛЮЧЕН.");
    thread::sleep(Duration::from_secs(2));

    //-------------------------------------------------//
    //--------------------_LocalData_------------------//
    main_window.set_global_wifi_status("online".into());
    log_to_lcdln!("Инициализация времени...");
    let sntp = EspSntp::new_default().unwrap();
    log_to_lcdln!("Ожидание точного времени...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        FreeRtos::delay_ms(500);
    }
    unsafe {
        std::env::set_var("TZ", "MSK-3");
        esp_idf_sys::tzset();
    }
    log_to_lcdln!("Время успешно синхронизировано!");
    std::mem::forget(sntp);
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

        let i2c_driver_clone = i2c_driver.clone();
        let mut bmp_holder = None;

        let mut timer_bmp_action = move || {
            let mut i2c_ref = i2c_driver_clone.borrow_mut();
            if bmp_holder.is_none() {
                println!("[BMP180] Первая инициализация датчика...");
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
                if let Some(ui) = ui_bmp.upgrade() {
                    ui.set_global_temp(format!("{:.1}", t).into());
                    let pressure_mm_hg = p as f64 / 133.3224;
                    let press_str = format!("{:.1} ", pressure_mm_hg);
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
                    buffer_size: Some(1024 * 8),
                    buffer_size_tx: Some(512 * 2),
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
    //---------------------------------------------------------//
    //---------------------_MQTT_Brocker_----------------------//
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:1883").unwrap();
        println!("MQTT Сервер успешно запущен на ESP32 и слушает порт 1883...");

        for stream in listener.incoming() {
            match stream {
                Ok(socket) => {
                    thread::spawn(move || {
                        if let Err(e) = handle_client(socket) {
                            println!("Сессия завершена с ошибкой: {:?}", e);
                        }
                    });
                }
                Err(e) => println!("TCP Error: {:?}", e),
            }
        }
    });
    //---------------------------------------------------------//
    let mut was_pressed = false;
    let mut release_debounce_counter = 0;
    let mut last_slint_pos = slint::LogicalPosition::new(0.0, 0.0);
    loop {
        display.handle_touch_pad(was_pressed, release_debounce_counter, last_slint_pos);
    }
}

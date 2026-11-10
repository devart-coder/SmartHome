use std::{
    io::Error,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use GroupOne::{
    mqtt_handler::Handler,
    relay::Relay,
    security::{PASSWORD, SDDI},
    wifi::Wifi,
};
use esp_idf_hal::{
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{log::EspLogger, wifi::AuthMethod};
use log::info;

fn main() -> anyhow::Result<()> {
    EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Info);
    let p = Peripherals::take().unwrap();
    // PinDriver::output(p.pins.gpio38)
    // .unwrap()
    // .set_high()
    // .unwrap();
    let i2c = I2cDriver::new(
        p.i2c0,
        p.pins.gpio42, // SDA
        p.pins.gpio41, // SCL
        &I2cConfig::default(),
    )
    .unwrap();
    let relay_ptr = Arc::new(Mutex::new(Relay::new(i2c)));
    //-------------------------------------------------//
    //---------------------_WIFI_----------------------//
    info!("Инициализация Wi-Fi...");
    let mut wifi = Wifi::new(p.modem);
    wifi.init_config(|b| {
        b.set_ssid(SDDI)
            .set_password(PASSWORD)
            .set_auth_method(AuthMethod::WPA2Personal)
    });
    let mut connections_retry: u8 = 0;
    while connections_retry <= 7
        && let Err(_) = wifi.show_connection(|w| {
            info!("Подключение к Wi-Fi... ");
            match w.start() {
                Ok(_) => {
                    info!("✅");
                }
                Err(err) => {
                    info!("Error {}", err);
                    connections_retry += 1;
                    return Err(err);
                }
            };
            info!("Поиск сети... ");
            match w.connect() {
                Ok(_) => {
                    info!("✅");
                    info!("Получение IP-адреса (DHCP)...");
                    match w.wait_netif_up() {
                        Ok(_) => {
                            if let Ok(ip_info) = w.wifi().sta_netif().get_ip_info() {
                                info!("✅\nIp: {:?}", ip_info.ip);
                            } else {
                                info!("✅ (No IP)");
                            }
                        }
                        Err(err) => {
                            connections_retry += 1;
                            info!("Error {}", err);
                            return Err(err);
                        }
                    };
                }
                Err(err) => {
                    info!("Error {}", err);
                    connections_retry += 1;
                    return Err(err);
                }
            };
            info!("Wi-Fi успешно подключен!");
            unsafe {
                esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
                info!("Режим энергосбережения Wi-Fi успешно ОТКЛЮЧЕН.");
            };
            Ok(())
        })
    {}
    //-------------------------------------------------//
    let listener = TcpListener::bind("0.0.0.0:1883").unwrap();
    info!("MQTT Сервер успешно запущен на ESP32 и слушает порт 1883...");
    for stream in listener.incoming() {
        match stream {
            Ok(socket) => {
                let timeout = Some(Duration::from_secs(10));
                if let Err(e) = socket.set_read_timeout(timeout) {
                    log::error!("Не удалось установить read timeout: {:?}", e);
                }
                if let Err(e) = socket.set_write_timeout(timeout) {
                    log::error!("Не удалось установить write timeout: {:?}", e);
                }
                let relay = relay_ptr.clone();
                thread::spawn(move || {
                    if let Err(e) = Handler::new().handle_client(socket, relay.clone()) {
                        log::error!("Сессия завершена с ошибкой: {:?}", e);
                    }
                });
            }
            Err(e) => log::error!("TCP Error: {:?}", e),
        }
    }
    Ok(())
}

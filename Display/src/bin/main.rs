use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::mqtt::client::MqttClientConfiguration;
use esp_idf_svc::mqtt::client::QoS;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::AuthMethod;
use log::{error, info};
use master_display::display::Display;
use master_display::mqtt_manager::MqttClientManager;
use master_display::security::LightSubscriptions;
use master_display::security::{BROKER_URL, PASSWORD, SDDI};
use master_display::switchers_group_2;
use master_display::ui::GlobalLightControllGroups;
use master_display::ui::{GlobalConnectionsStatus, MainWindow};
use master_display::wifi::Wifi;
use master_display::{bmp_block, timer_block, to_lcd, to_lcdln, wifi_step};
use master_display::{owners_group_2, sntp};
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
    let ui = MainWindow::new().unwrap();
    ui.set_is_loading(true);
    //-------------------------------------------------//
    //---------------------_WIFI_----------------------//
    to_lcdln!("Инициализация Wi-Fi...", display, ui);
    let mut wifi = Wifi::new(p.modem);
    wifi.init_config(|b| {
        b.set_ssid(SDDI)
            .set_password(PASSWORD)
            .set_auth_method(AuthMethod::WPA2Personal)
    });
    wifi.show_connection(|w| {
        let _ = wifi_step!("Подключение к Wi-Fi... ", w.start(), display, ui);
        if wifi_step!("Поиск сети...", w.connect(), display, ui).is_ok() {
            if wifi_step!(
                "Получение IP-адреса (DHCP)...",
                w.wait_netif_up(),
                display,
                ui
            )
            .is_ok()
            {
                if let Ok(ip_info) = w.wifi().sta_netif().get_ip_info() {
                    to_lcdln!(format!("Ip: {:?}", ip_info.ip).as_str(), display, ui);
                    unsafe {
                        esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
                        info!("Режим энергосбережения Wi-Fi успешно ОТКЛЮЧЕН.");
                    }
                    to_lcdln!("Wi-Fi успешно подключен!", display, ui);
                    ui.global::<GlobalConnectionsStatus>()
                        .set_wifi_status("online".into());
                    //--------------------_LocalData_------------------//
                    to_lcdln!("Инициализация времени...", display, ui);
                    sntp!("MSK-3", 10, display, ui);
                }
            };
        }
    });
    ui.set_is_loading(false);
    //-------------------------------------------------//
    let _timer = timer_block(ui.as_weak());
    let _ui_timer_bmp = bmp_block(ui.as_weak(), i2c_driver.clone());
    display.update();
    //-------------------------------------------------//
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
    switchers_group_2!(client, ui);
    owners_group_2!(client, ui);

    let uiw = ui.as_weak();
    let client = client.clone();
    ui.global::<GlobalConnectionsStatus>()
        .on_mqtt_button_pressed(move || {
            let current_mqtt_status = uiw
                .unwrap()
                .global::<GlobalConnectionsStatus>()
                .get_mqtt_status();
            if current_mqtt_status == "disconnected" {
                uiw.unwrap()
                    .global::<GlobalConnectionsStatus>()
                    .set_mqtt_status("connected".into());
                client.lock().unwrap().restart();
            } else if current_mqtt_status == "connected" {
                uiw.unwrap()
                    .global::<GlobalConnectionsStatus>()
                    .set_mqtt_status("disconnected".into());
            }
        });
    //---------------------------------------------------------//
    loop {
        display.handle_touch_pad();
    }
}

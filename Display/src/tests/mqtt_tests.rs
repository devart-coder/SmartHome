use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::mqtt::client::{Configuration as MqttClientConfig, EspMqttClient, MqttEvent, QoS};
use esp_idf_svc::wifi::{AccessPointConfiguration, Configuration, EspWifi};
use rumqttd::{Broker, Config};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let sys_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;

    // 1. Создаем собственную Wi-Fi сеть (Точка доступа)
    let mut wifi = EspWifi::new(peripherals.modem, sys_loop, Some(nvs))?;
    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: "ESP32_LCD_Net".try_into().unwrap(),
        password: "Password123".try_into().unwrap(),
        channel: 6,
        ..Default::default()
    }))?;
    wifi.start()?;
    println!("Wi-Fi AP запущена! IP брокера: 192.168.4.1");

    // 2. Запуск MQTT-брокера Rumqttd в отдельном потоке
    thread::spawn(|| {
        println!("Запуск MQTT брокера...");
        // Создаем базовую конфигурацию (по умолчанию слушает порт 1883)
        let config = Config::default();
        let mut broker = Broker::new(config);

        if let Err(e) = broker.start() {
            println!("Ошибка брокера: {:?}", e);
        }
    });

    // Даем брокеру секунду, чтобы подняться
    thread::sleep(Duration::from_secs(1));

    // 3. Подключаем ЛОКАЛЬНЫЙ MQTT-клиент для ЭТОГО же экрана (LCD №1)
    let client_config = MqttClientConfig::default();
    // Обратите внимание: подключаемся к самому себе (127.0.0.1 или 192.168.4.1)
    let mut local_client =
        EspMqttClient::new("mqtt://127.0.0.1:1883", &client_config, move |event| {
            if let MqttEvent::Received(message) = event {
                let payload = std::str::from_utf8(message.data()).unwrap_or("");
                if message.topic() == "home/relay1/state" {
                    // ТУТ КОД ОБНОВЛЕНИЯ ИНТЕРФЕЙСА ДЛЯ ЭКРАНА №1
                    println!("Экран 1: статус реле изменился на {}", payload);
                }
            }
        })?;

    // Подписываемся на статус реле, чтобы экран знал, включено оно или выключено
    local_client.subscribe("home/relay1/state", QoS::AtMostOnce)?;

    // 4. Основной графический цикл (Отрисовка экрана LCD №1)
    loop {
        // Логика отрисовки графики (LVGL, Slint или embedded-graphics)

        // Пример отправки команды при клике по экрану:
        // local_client.publish("home/relay1/cmd", QoS::AtMostOnce, false, b"TOGGLE").unwrap();

        thread::sleep(Duration::from_millis(20));
    }
}

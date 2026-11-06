use std::net::Ipv4Addr;

use anyhow::Error;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    ipv4::{self, Configuration, RouterConfiguration},
    netif::{EspNetif, NetifConfiguration, NetifStack},
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, EspWifi, WifiDriver},
};
use esp_idf_sys::EspError;

use crate::security::netif_config;

#[derive(Debug, Default)]
pub struct WifiConfigBuilder<'a> {
    ssid: &'a str,
    password: &'a str,
    auth_method: AuthMethod,
}
impl<'a> WifiConfigBuilder<'a> {
    pub fn builder() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_ssid(mut self, ssid: &'a str) -> Self {
        self.ssid = ssid;
        self
    }

    pub fn set_password(mut self, password: &'a str) -> Self {
        self.password = password;
        self
    }

    pub fn set_auth_method(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }
    pub fn build(self) -> ClientConfiguration {
        ClientConfiguration {
            ssid: self.ssid.try_into().unwrap_or_default(),
            auth_method: self.auth_method.try_into().unwrap_or_default(),
            password: self.password.try_into().unwrap_or_default(),
            ..Default::default()
        }
    }
}

pub struct Wifi<'a> {
    wifi: BlockingWifi<EspWifi<'a>>,
}
impl<'a> Wifi<'a> {
    pub fn new(modem: Modem<'static>) -> Self {
        let nvs = EspDefaultNvsPartition::take().unwrap();
        let sys_loop = EspSystemEventLoop::take().unwrap();

        // 2. Инициализируем сетевой интерфейс с нашей конфигурацией
        let netif = EspNetif::new_with_conf(&netif_config()).unwrap();
        let wifi_driver = WifiDriver::<'a>::new(modem, sys_loop.clone(), Some(nvs)).unwrap();
        let esp_wifi =
            EspWifi::wrap_all(wifi_driver, netif, EspNetif::new(NetifStack::Ap).unwrap()).unwrap();
        let wifi = BlockingWifi::wrap(esp_wifi, sys_loop).unwrap();
        Self { wifi: wifi }
    }
    pub fn init_config<F>(&mut self, f: F)
    where
        F: Fn(WifiConfigBuilder) -> WifiConfigBuilder,
    {
        let config = f(WifiConfigBuilder::builder()).build();
        self.wifi
            .set_configuration(&esp_idf_svc::wifi::Configuration::Client(
                ClientConfiguration {
                    ssid: config.ssid.try_into().unwrap(),
                    password: config.password.try_into().unwrap(),
                    auth_method: config.auth_method,
                    ..config
                },
            ))
            .unwrap();
    }
    pub fn connection(&mut self) {
        println!("Подключение к Wi-Fi...");
        self.wifi.start().unwrap();
        println!("Поиск сети...");
        self.wifi.connect().unwrap();
        println!("Сеть найдена! Получение IP-адреса (DHCP)...");
        self.wifi.wait_netif_up().unwrap();
        println!("Wi-Fi успешно подключен!");
    }
    pub fn show_connection<F>(&mut self, connect: F) -> Result<(), EspError>
    where
        F: FnOnce(&mut BlockingWifi<EspWifi>) -> Result<(), EspError>,
    {
        connect(&mut self.wifi)
    }
}

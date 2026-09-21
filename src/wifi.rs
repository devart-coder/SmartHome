use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, EspWifi},
};

#[derive(Debug, Default)]
pub struct WifiConfigBuilder {
    ssid: String,
    password: String,
    auth_method: AuthMethod,
}
impl WifiConfigBuilder {
    pub fn builder() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_ssid(mut self, ssid: String) -> Self {
        self.ssid = ssid;
        self
    }

    pub fn set_password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn set_auth_method(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }
    pub fn build(self) -> ClientConfiguration {
        ClientConfiguration {
            ssid: self.ssid.as_str().try_into().unwrap(),
            auth_method: self.auth_method.try_into().unwrap(),
            password: self.password.as_str().try_into().unwrap(),
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
        let esp_wifi = EspWifi::<'a>::new(modem, sys_loop.clone(), Some(nvs)).unwrap();
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
    pub fn show_connection<F>(&mut self, connect: F)
    where
        F: FnOnce(&mut BlockingWifi<EspWifi>),
    {
        connect(&mut self.wifi);
    }
}

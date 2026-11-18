use crate::subscriptions;
pub const BROKER_URL: &str = "";
pub const SDDI: &str = "";
pub const PASSWORD: &str = "";
subscriptions!(ControllRoom, Restroom, Bathroom, Breakroom,);
pub fn netif_config() -> NetifConfiguration {
    let mut netif_config = NetifConfiguration::wifi_default_client();
    netif_config.ip_configuration = Some(ipv4::Configuration::Client(
        ipv4::ClientConfiguration::Fixed(ipv4::ClientSettings {
            ip: Ipv4Addr::new(_, _, _, _), // Желаемый IP устройства
            subnet: ipv4::Subnet {
                gateway: Ipv4Addr::new(_, _, _, _), // Шлюз (IP роутера)
                mask: ipv4::Mask(_),                // Маска
            },
            dns: Some(Ipv4Addr::new(_, _, _, _)), // Основной DNS
            secondary_dns: Some(Ipv4Addr::new(_, _, _, _)),
        }),
    ));
    netif_config
}

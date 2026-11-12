use crate::security::LightSubscriptions;
use crate::ui::GlobalLightControllGroups;
use esp_idf_svc::mqtt::client::QoS;
use mqttrs::QoS as OtherQoS;
#[macro_export]
macro_rules! to_lcd {
    ($msg:expr,$display:ident,$ui:ident) => {
        info!("{}", $msg);
        let mut local_str = $ui.get_boot_logs();
        local_str.push_str($msg);
        $ui.set_boot_logs(local_str);
        $display.update();
    };
}
#[macro_export]
macro_rules! to_lcdln {
    ($msg:expr,$display:ident,$ui:ident) => {
        let e = $msg.to_string() + "\n";
        to_lcd!(e.as_str(), $display, $ui)
    };
}
#[macro_export]
macro_rules! sntp {
    ($tz:expr, $timeout_sec:expr, $display:ident, $main_window:ident) => {{
        if let Ok(sntp) = esp_idf_svc::sntp::EspSntp::new_default() {
            let mut counter = 0;
            while sntp.get_sync_status() != esp_idf_svc::sntp::SyncStatus::Completed {
                esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
                counter += 1;
                if counter > $timeout_sec {
                    break;
                }
            }

            match sntp.get_sync_status() {
                esp_idf_svc::sntp::SyncStatus::Completed => {
                    unsafe {
                        std::env::set_var("TZ", $tz);
                        esp_idf_sys::tzset();
                    }
                    to_lcd!("Время успешно синхронизировано", $display, $main_window);
                    true
                }
                _ => {
                    to_lcd!("Ошибка синхронизации времени", $display, $main_window);
                    false
                }
            }
        } else {
            to_lcd!("Ошибка инициализации SNTP", $display, $main_window);
            false
        }
    }};
}

#[macro_export]
macro_rules! switchers_group_2 {
    ($client_guard:ident, $ui:ident) => {
        let manager_guard = $client_guard.clone();
        let ui = $ui.as_weak();
        ui.unwrap()
            .global::<GlobalLightControllGroups>()
            .on_g2_status(move |name, status| {
                #[cfg(Debug)]
                {
                    info!("Name : {} Status:{}", name, status);
                }
                let manager_guard = manager_guard.lock().unwrap();
                let client_arc = manager_guard.guard_client();
                let mut inner_guard = client_arc.lock().unwrap();
                let c_lock = inner_guard.as_mut().unwrap();
                let topic_name = if name == LightSubscriptions::ControllRoom.name() {
                    Some(LightSubscriptions::ControllRoom.topic())
                } else if name == LightSubscriptions::Restroom.name() {
                    Some(LightSubscriptions::Restroom.topic())
                } else if name == LightSubscriptions::Bathroom.name() {
                    Some(LightSubscriptions::Bathroom.topic())
                } else if name == LightSubscriptions::Breakroom.name() {
                    Some(LightSubscriptions::Breakroom.topic())
                } else {
                    None
                };
                if let Some(topic) = topic_name {
                    c_lock
                        .publish(
                            topic.as_str(),
                            QoS::AtLeastOnce,
                            true,
                            if status { b"on" } else { b"off" },
                        )
                        .unwrap();
                };
            });
    };
}
#[macro_export]
macro_rules! owners_group_2 {
    ($client_guard:ident, $ui:ident) => {
        let manager_guard = $client_guard.clone();
        let ui = $ui.as_weak();
        ui.unwrap()
            .global::<GlobalLightControllGroups>()
            .on_g2_ownership(move |name, status| {
                #[cfg(Debug)]
                {
                    info!("Name : {} Status:{}", name, status);
                }
                let manager_guard = manager_guard.lock().unwrap();
                let client_arc = manager_guard.guard_client();
                let mut inner_guard = client_arc.lock().unwrap();
                let c_lock = inner_guard.as_mut().unwrap();
                let topic_name = if name == LightSubscriptions::ControllRoom.name() {
                    Some(LightSubscriptions::ControllRoom.topic_with_ownership())
                } else if name == LightSubscriptions::Restroom.name() {
                    Some(LightSubscriptions::Restroom.topic_with_ownership())
                } else if name == LightSubscriptions::Bathroom.name() {
                    Some(LightSubscriptions::Bathroom.topic_with_ownership())
                } else if name == LightSubscriptions::Breakroom.name() {
                    Some(LightSubscriptions::Breakroom.topic_with_ownership())
                } else {
                    None
                };
                if let Some(topic) = topic_name {
                    c_lock
                        .publish(
                            topic.as_str(),
                            QoS::AtLeastOnce,
                            true,
                            if status { b"on" } else { b"off" },
                        )
                        .unwrap();
                };
            });
    };
}

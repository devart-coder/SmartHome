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

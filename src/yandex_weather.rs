use chrono::{Datelike, Weekday};
use esp_idf_svc::http::{Method, client::EspHttpConnection};
use slint::{SharedString, ToSharedString};
use tinyjson::JsonValue;

#[derive(Debug, Clone, Default)]
pub struct YandexWeatherFact {
    //Общее
    //Время суток на метеостанции: "d" — день, "n" — ночь.
    pub daytime: SharedString,
    //Текущее время года
    pub season: SharedString,
    //Текстовый код состояния погоды
    pub condition: SharedString,
    //Код иконки
    pub icon: SharedString,

    //Температура-Давление-Влажность
    //Фактическая температура воздуха прямо сейчас
    pub temp: SharedString,
    //Ощущаемая температура воздуха с учетом влажности и ветра
    pub feels_like: SharedString,
    //Давление Влажность
    // Атмосферное давление в мм ртутного столба (762 мм рт. ст.). Самый привычный нам формат.
    pub pressure_mm: SharedString,
    //Атмосферное давление в гектопаскалях (1015 гПа).
    pub pressure_pa: SharedString,
    //Относительная влажность воздуха в процентах (88%).
    pub humidity: SharedString,

    //Ветер
    //Скорость ветра в метрах в секунду (3.1 м/с).
    pub wind_speed: SharedString,
    //Скорость порывов ветра (6.7 м/с).
    pub wind_gust: SharedString,
    //Направление ветра строкой (Юго-Западный).
    pub wind_dir: SharedString,

    //Осадки
    //Степень облачности в долях единицы (*100%)
    pub cloudness: SharedString,
    //Тип осадков: 0 — без осадков, 1 — дождь, 2 — дождь со снегом, 3 — снег, 4 — град.
    pub prec_type: SharedString,
    //Вероятность выпадения осадков в процентах (0%)
    pub prec_prob: SharedString,
    //Есть ли гроза прямо сейчас (Нет
    pub is_thunder: SharedString,
}
impl YandexWeatherFact {
    pub fn connection(
        client: &mut EspHttpConnection,
        api_key: &str,
        lat: &str,
        lon: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.weather.yandex.ru/v2/forecast?lat={}&lon={}&lang=ru_RU&hours=false&limit=1&extra=false",
            lat, lon
        );
        let headers = [
            ("X-Yandex-Weather-Key", api_key),
            ("Accept", "application/json"),
            ("Connection", "close"),
        ];
        let _ = client.initiate_request(Method::Get, &url, &headers)?;
        let _ = client.initiate_response()?;

        if client.status() != 200 {
            return Err(format!("Ошибка Яндекс Погоды. Код: {}", client.status()).into());
        }

        // 4. Считываем данные в буфер
        let mut buf = [0u8; 32768];
        let mut total_bytes = 0;

        while let Ok(bytes_read) = client.read(&mut buf[total_bytes..]) {
            if bytes_read == 0 {
                break;
            }
            total_bytes += bytes_read;
        }

        let body_str = std::str::from_utf8(&buf[..total_bytes])?;
        Ok(body_str.to_string())
    }
    pub fn parse(string: String) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed: JsonValue = string.parse()?;
        if let JsonValue::Object(fact) = &parsed["fact"] {
            let temp = fact
                .get("temp")
                .and_then(|v| match v {
                    JsonValue::Number(n) => Some((*n as i32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let feels_like = fact
                .get("feels_like")
                .and_then(|v| match v {
                    JsonValue::Number(n) => Some((*n as i32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let condition = fact
                .get("condition")
                .and_then(|v| match v {
                    JsonValue::String(s) => Some(s.clone().into()),
                    _ => None,
                })
                .unwrap_or_default();
            let wind_speed = fact
                .get("wind_speed")
                .and_then(|v| match v {
                    JsonValue::Number(n) => Some((*n as f32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let wind_gust = fact
                .get("wind_gust")
                .and_then(|v| match v {
                    JsonValue::Number(n) => Some((*n as f32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let wind_dir = fact
                .get("wind_dir")
                .and_then(|v| match v {
                    JsonValue::String(s) => Some(s.clone().into()),
                    _ => None,
                })
                .unwrap_or_default();
            let pressure_mm = fact
                .get("pressure_mm")
                .and_then(|v| match v {
                    JsonValue::Number(n) => Some((*n as i32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let humidity = fact
                .get("humidity")
                .and_then(|v| match v {
                    JsonValue::Number(n) => Some((*n as i32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let daytime = fact
                .get("daytime")
                .and_then(|v| match v {
                    JsonValue::String(s) => Some(s.clone().into()),
                    _ => None,
                })
                .unwrap_or_default();
            let season = fact
                .get("season")
                .and_then(|v| match v {
                    JsonValue::String(s) => Some(s.clone().into()),
                    _ => None,
                })
                .unwrap_or_default();
            let pressure_pa = fact
                .get("pressure_pa")
                .and_then(|v| match v {
                    JsonValue::Number(s) => Some((*s as i32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let cloudness = fact
                .get("cloudness")
                .and_then(|v| match v {
                    JsonValue::Number(s) => Some((*s as f32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let prec_type = fact
                .get("prec_type")
                .and_then(|v| match v {
                    JsonValue::Number(s) => Some(YandexWeatherFact::map_prec_type(*s as i32)),
                    _ => None,
                })
                .unwrap_or("--".into());
            let prec_prob = fact
                .get("prec_prob")
                .and_then(|v| match v {
                    JsonValue::Number(s) => Some((*s as i32).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let is_thunder = fact
                .get("is_thunder")
                .and_then(|v| match v {
                    JsonValue::Boolean(s) => Some((*s).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            let icon = fact
                .get("icon")
                .and_then(|v| match v {
                    JsonValue::String(s) => Some((s.clone()).to_shared_string()),
                    _ => None,
                })
                .unwrap_or("--".into());
            return Ok(YandexWeatherFact {
                temp,
                feels_like,
                condition,
                wind_speed,
                wind_gust,
                wind_dir,
                pressure_mm,
                humidity,
                daytime,
                season,
                pressure_pa,
                cloudness,
                prec_type,
                prec_prob,
                is_thunder,
                icon,
            });
        }
        Err("Не удалось найти блок 'fact'".into())
    }
    pub fn map_prec_type(prec_type: i32) -> SharedString {
        match prec_type {
            0 => "Без осадков".to_shared_string(),
            1 => "Дождь".to_shared_string(),
            2 => "Дождь со снегом".to_shared_string(),
            3 => "Снег".to_shared_string(),
            4 => "Град".to_shared_string(),
            _ => "--".to_shared_string(),
        }
    }
    pub fn get_rus_weekday(now: &chrono::DateTime<chrono::Local>) -> &'static str {
        match now.weekday() {
            Weekday::Mon => "Понедельник",
            Weekday::Tue => "Вторник",
            Weekday::Wed => "Среда",
            Weekday::Thu => "Четверг",
            Weekday::Fri => "Пятница",
            Weekday::Sat => "Суббота",
            Weekday::Sun => "Воскресенье",
        }
    }
    pub fn translate_condition(code: &SharedString) -> SharedString {
        match code.as_str() {
            "clear" => "Ясно".to_shared_string(),
            "partly-cloudy" => "Малооблачно".to_shared_string(),
            "cloudy" => "Облачно".to_shared_string(),
            "overcast" => "Пасмурно".to_shared_string(),
            "light-rain" => "Небольшой дождь".to_shared_string(),
            "rain" => "Дождь".to_shared_string(),
            "heavy-rain" => "Сильный дождь".to_shared_string(),
            "showers" => "Ливень".to_shared_string(),
            "wet-snow" => "Дождь со снегом".to_shared_string(),
            "light-snow" => "Небольшой снег".to_shared_string(),
            "snow" => "Снег".to_shared_string(),
            "snow-showers" => "Снегопад".to_shared_string(),
            "hail" => "Град".to_shared_string(),
            "thunderstorm" => "Гроза".to_shared_string(),
            "thunderstorm-with-rain" => "Дождь с грозой".to_shared_string(),
            "thunderstorm-with-hail" => "Гроза с градом".to_shared_string(),
            _ => code.to_shared_string(), // Если код неизвестен, возвращаем его как есть
        }
    }
    pub fn translate_wind_dir(dir: &SharedString) -> SharedString {
        match dir.as_str() {
            "nw" => "↖".to_shared_string(),
            "n" => "↑".to_shared_string(),
            "ne" => "↗".to_shared_string(),
            "e" => "→".to_shared_string(),
            "se" => "↘".to_shared_string(),
            "s" => "↓".to_shared_string(),
            "sw" => "↙".to_shared_string(),
            "w" => "←".to_shared_string(),
            "c" => "☉".to_shared_string(),
            _ => dir.to_shared_string(),
        }
    }
}

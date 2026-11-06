
use chrono::{Datelike, Weekday};
use esp_idf_svc::http::{Method, client::EspHttpConnection};
use log::info;
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
            "https://api.weather.yandex.ru/v2/forecast?lat={}&lon={}&lang=ru_RU&hours=false&extra=false",
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
        let mut buf = vec![0u8; 32768];
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
    pub fn parse(string: &String) -> Result<Self, Box<dyn std::error::Error>> {
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

#[derive(Debug, Default)]
pub struct YandexWeatherForeCasts {
    today: ForeCastsData,
    tomorrow: ForeCastsData,
}
impl<'a> YandexWeatherForeCasts {
    pub fn parse(
        string: &String,
    ) -> Result<(ForeCastsData, ForeCastsData), Box<dyn std::error::Error>> {
        let mut answer = (ForeCastsData::default(), ForeCastsData::default());
        let parsed_string: JsonValue = string.parse()?;
        println!("{:#?}", parsed_string);
        if let JsonValue::Array(result) = &parsed_string["forecasts"] {
            info!("forecasts_len: {}", result.len());
            let closer = |value: &JsonValue| match value {
                JsonValue::Object(value) => {
                    let daytime = value
                        .get("daytime")
                        .and_then(|d| match d {
                            JsonValue::String(d) => Some(d.to_shared_string()),
                            _ => None,
                        })
                        .unwrap_or("-".to_shared_string());
                    let cloudness = value
                        .get("cloudness")
                        .and_then(|d| match d {
                            JsonValue::Number(d) => Some(*d as f32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let condition = value
                        .get("condition")
                        .and_then(|c| match c {
                            JsonValue::String(c) => Some(c.to_shared_string()),
                            _ => None,
                        })
                        .unwrap_or("--".to_shared_string());
                    let feels_like = value
                        .get("feels_like")
                        .and_then(|fl| match fl {
                            JsonValue::Number(fl) => Some(*fl as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let humidity = value
                        .get("humidity")
                        .and_then(|fl| match fl {
                            JsonValue::Number(fl) => Some(*fl as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let polar = value
                        .get("polar")
                        .and_then(|fl| match fl {
                            JsonValue::Boolean(fl) => Some(*fl),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let prec_period = value
                        .get("prec_period")
                        .and_then(|pp| match pp {
                            JsonValue::Number(pp) => Some(*pp as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let prec_strength = value
                        .get("prec_strength")
                        .and_then(|ps| match ps {
                            JsonValue::Number(ps) => Some(*ps as f32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let prec_type = value
                        .get("prec_type")
                        .and_then(|ps| match ps {
                            JsonValue::Number(ps) => Some(*ps as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let temp_avg = value
                        .get("temp_avg")
                        .and_then(|ta| match ta {
                            JsonValue::Number(ta) => Some(*ta as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let temp_max = value
                        .get("temp_max")
                        .and_then(|tm| match tm {
                            JsonValue::Number(tm) => Some(*tm as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let temp_min = value
                        .get("temp_min")
                        .and_then(|tm| match tm {
                            JsonValue::Number(tm) => Some(*tm as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let uv_index = value
                        .get("uv_index")
                        .and_then(|tm| match tm {
                            JsonValue::Number(tm) => Some(*tm as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let wind_angle = value
                        .get("wind_angle")
                        .and_then(|wa| match wa {
                            JsonValue::Number(wa) => Some(*wa as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let wind_dir = value
                        .get("wind_dir")
                        .and_then(|wd| match wd {
                            JsonValue::String(wd) => Some(wd.to_shared_string()),
                            _ => None,
                        })
                        .unwrap_or("--".to_shared_string());
                    let wind_gust = value
                        .get("wind_gust")
                        .and_then(|wg| match wg {
                            JsonValue::Number(wg) => Some(*wg as u32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let wind_speed = value
                        .get("wind_speed")
                        .and_then(|ws| match ws {
                            JsonValue::Number(ws) => Some(*ws as f32),
                            _ => None,
                        })
                        .unwrap_or_default();
                    Some(CastsParts {
                        cloudness,
                        condition,
                        daytime,
                        feels_like,
                        humidity,
                        polar,
                        prec_period,
                        prec_strength,
                        prec_type,
                        temp_avg,
                        temp_max,
                        temp_min,
                        uv_index,
                        wind_angle,
                        wind_dir,
                        wind_gust,
                        wind_speed,
                    })
                }
                _ => None,
            };
            if let JsonValue::Object(today_element) = &result[0] {
                let sun_rise = today_element
                    .get("sunrise")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let sun_set = today_element
                    .get("sunset")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let rise_begin = today_element
                    .get("rise_begin")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let set_end = today_element
                    .get("set_end")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let parts = today_element
                    .get("parts")
                    .and_then(|v| match v {
                        JsonValue::Object(v) => {
                            let mut parts = CastsPartsData::default();
                            parts.day = v.get("day").and_then(closer).unwrap_or_default();
                            parts.day_short =
                                v.get("day_short").and_then(closer).unwrap_or_default();
                            parts.evening = v.get("evening").and_then(closer).unwrap_or_default();
                            parts.morning = v.get("morning").and_then(closer).unwrap_or_default();
                            parts.night = v.get("night").and_then(closer).unwrap_or_default();
                            parts.night_short =
                                v.get("night_short").and_then(closer).unwrap_or_default();
                            Some(parts)
                        }
                        _ => None,
                    })
                    .unwrap_or_default();
                answer.0 = ForeCastsData {
                    rise_begin,
                    set_end,
                    sun_rise,
                    sun_set,
                    parts,
                    ..Default::default()
                };
            }
            if let JsonValue::Object(tomorrow_element) = &result[1] {
                let sun_rise = tomorrow_element
                    .get("sunrise")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let sun_set = tomorrow_element
                    .get("sunset")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let rise_begin = tomorrow_element
                    .get("rise_begin")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let set_end = tomorrow_element
                    .get("set_end")
                    .and_then(|v| match v {
                        JsonValue::String(v) => Some(v.to_shared_string()),
                        _ => None,
                    })
                    .unwrap_or("--:--".to_shared_string());
                let parts = tomorrow_element
                    .get("parts")
                    .and_then(|v| match v {
                        JsonValue::Object(v) => {
                            let mut parts = CastsPartsData::default();
                            parts.day = v.get("day").and_then(closer).unwrap_or_default();
                            parts.day_short =
                                v.get("day_short").and_then(closer).unwrap_or_default();
                            parts.evening = v.get("evening").and_then(closer).unwrap_or_default();
                            parts.morning = v.get("morning").and_then(closer).unwrap_or_default();
                            parts.night = v.get("night").and_then(closer).unwrap_or_default();
                            parts.night_short =
                                v.get("night_short").and_then(closer).unwrap_or_default();
                            Some(parts)
                        }
                        _ => None,
                    })
                    .unwrap_or_default();
                answer.1 = ForeCastsData {
                    rise_begin,
                    set_end,
                    sun_rise,
                    sun_set,
                    parts,
                    ..Default::default()
                };
            }
            return Ok(answer);
        }
        Err("[Error]: Parse failed.".into())
    }
}

#[derive(Debug, Default)]
pub struct ForeCastsData {
    //Время начала восхода Солнца,
    rise_begin: SharedString,
    //Время окончания восхода Солнца
    sun_rise: SharedString,
    //Время начала заката Солнца
    sun_set: SharedString,
    //Время окончания заката Солнца
    set_end: SharedString,
    //Прогнозы по времени суток и 12-часовые прогнозы.
    parts: CastsPartsData,
    //
    hourse: Vec<CastsHourse>,
}

#[derive(Debug, Default)]
pub struct CastsPartsData {
    //прогноз на день
    day: CastsParts,
    //12-часовой прогноз на день.
    day_short: CastsParts,
    //прогноз на вечер.
    evening: CastsParts,
    //прогноз на утро
    morning: CastsParts,
    //прогноз на ночь
    night: CastsParts,
    //прогноз на ночь, для которого исключены поля temp_min и temp_max,
    //в поле temp указывается минимальная температура за ночной период.
    night_short: CastsParts,
}

#[derive(Debug, Default)]
pub struct CastsParts {
    // Светлое или темное время суток.
    daytime: SharedString,
    // Облачность
    cloudness: f32,
    // Код расшифровки погодного описания
    condition: SharedString,
    // Влажность воздуха (в процентах).
    humidity: u32,
    // Признак того, что время суток, указанное в поле daytime, является полярным.
    polar: bool,
    // Прогнозируемый период осадков (в минутах).
    prec_period: u32,
    // Сила осадков.
    prec_strength: f32,
    // Тип осадков.
    prec_type: u32,
    // Средняя температура для времени суток (°C).
    temp_avg: u32,
    //Максимальная температура для времени суток (°C).
    temp_max: u32,
    //Минимальная температура для времени суток (°C).
    temp_min: u32,
    // Ощущаемая температура (°C).
    feels_like: u32,
    uv_index: u32,
    wind_angle: u32,
    // Направление ветра.
    wind_dir: SharedString,
    // Скорость порывов ветра (в м/с).
    wind_gust: u32,
    // Скорость ветра (в м/с).
    wind_speed: f32,
}

#[derive(Debug, Default)]
pub struct CastsHourse {
    hour: SharedString,
    cloudness: f32,
    condition: SharedString,
    feels_like: u32,
    humidity: u32,
    is_thunder: bool,
    prec_period: u32,
    prec_strength: u32,
    prec_type: u32,
    temp: u32,
    uv_index: u32,
    wind_angle: u32,
    wind_dir: SharedString,
    wind_gust: f32,
    wind_speed: f32,
}

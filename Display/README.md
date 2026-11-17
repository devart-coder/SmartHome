# Display
  - Модуль проекта SmartHome предназначен для визуального отображения пользовательского интерфейса (UI).

## Структура модуля:

```text
Display/
├── .cargo/
│   └── config.toml          # Конфигурация компиляции и таргета (ESP32)
├── src/                     # Исходный код проекта
│   ├── bin/
│   │   └── main.rs          # Главный исполняемый файл (точка входа)
│   ├── lib.rs               # Основная логика, подключение модулей и макросов
│   ├── display.rs           # Модуль работы с дисплеем
│   ├── macros.rs            # Вспомогательные макросы
│   ├── wifi.rs              # Модуль подключения и работы с Wi-Fi
│   ├── yandex_weather.rs    # Интеграция с сервисом «Яндекс Погода»
│   └── mqtt_manager.rs      # Модуль работы с MQTT-клиентом
├── ui/                      # Компоненты пользовательского интерфейса (Slint)
│   ├── global/              # Глобальные переменные и состояния UI
│   ├── icons/               # Иконки интерфейса
│   ├── main_tab/            # Вкладка «Главная»
│   ├── pictures/            # Изображения и графические ресурсы
│   ├── weather_tab/         # Вкладка «Погода»
│   └── main_window.slint    # Главный экран приложения
├── build.rs                 # Скрипт сборки и компиляции Slint для ESP32
├── Cargo.toml               # Конфигурация крейта и зависимости
├── sdkconfig.defaults       # Параметры конфигурации ESP-IDF по умолчанию
├── LICENCE                  # Лицензия проекта
└── README.md                # Документация проекта 
```

## Установка и запуск

### 1. Предпросмотр интерфейса на ПК (опционально)

Установите утилиту `slint-viewer` для запуска и отладки верстки без прошивки микроконтроллера:

```bash
cargo install slint-viewer
```

Запуск окна предпросмотра:

```bash
slint-viewer ui/main_window.slint
```

### 3. Настройка констант и перечислений для запуска проекта:

* **Создайте файл security.rs. Добавить код из примера и актуализировать информацию для своего проекта:**:
  **Пример:**
  ```text
  //URL брокера
  pub const BROKER_URL: &str = "";
  //Логин wifi сети
  pub const SDDI: &str = "";
  //Пароль wifi сети
  pub const PASSWORD: &str = "";

  pub enum LightSubscriptions {
      //Добавьте собственные перечисления или раскомментируйте описание ниже
      //ControllRoom,
      //Restroom,
      //Bathroom,
      //Breakroom,
  }
  impl LightSubscriptions {
      pub fn name(&self) -> &'static str {
          match self {
            //Добавьте собственную реализацию или раскомментируйте описание ниже
            //Self::ControllRoom => "controllroom",
            //Self::Restroom => "restroom",
            //Self::Bathroom => "bathroom",
            //Self::Breakroom => "breakroom",
          }
      }
      pub fn topic(&self) -> String {
          //Добавьте собственную реализацию или раскомментируйте описание ниже
          //match self {
          //    _ => format!("свой/полный/путь/топика/{}", self.name()),
          //}
          //После реализации закомменируйте или удалите код ниже
          #[cfg(not(rust_analyzer))]
          compile_error!("Необходимо реализовать метод topic в файле security.rs");
          unreachable!()
      }
      pub fn topic_with_ownership(&self) -> String {
          //Добавьте собственную реализацию или раскомментируйте описание ниже
          //format!("{}/light_ownership", self.topic())
          //После реализации закомменируйте или удалите код ниже
          #[cfg(not(rust_analyzer))]
          compile_error!("Необходимо реализовать метод topic_with_ownership в файле security.rs");
          unreachable!()
      }
  }
  ```

### 3. Сборка проекта

* **Debug-сборка:**
  ```bash
  cargo build
  ```
* **Release-сборка:**
  ```bash
  cargo build --release
  ```

### 4. Прошивка бинарного файла в ESP32

Сборка и загрузка прошивки на подключенную плату (требуется установленный раннер `espflash` или `cargo-espflash`):

* **Debug-загрузка:**
  ```bash
  cargo run
  ```
* **Release-загрузка:**
  ```bash
  cargo run --release
  ```

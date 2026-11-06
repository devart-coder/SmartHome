use anyhow::Result;
use esp_idf_sys as idf; // Низкоуровневое SDK Espressif

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::peripherals::Peripherals;
use log::set_max_level;

// Размеры экрана Waveshare 7 дюймов
const WIDTH: usize = 800;
const HEIGHT: usize = 480;

fn main() -> Result<()> {
    // Системная инициализация
    esp_idf_sys::link_patches();
    esp_idf_logger::init()?;
    set_max_level(log::LevelFilter::Info);

    println!("============= [RUST APP START (ESP-IDF v5.2.2)] =============");

    println!("1. Инициализация периферии...");
    let peripherals = Peripherals::take().unwrap();

    // Включение подсветки дисплея (GPIO 10 на Waveshare)

    // 2. Включение подсветки дисплея (поддерживает разные ревизии плат Waveshare)
    println!("2. Пробуем включить подсветку дисплея...");

    // Проверяем стандартный GPIO 10
    if let Ok(mut backlight10) = esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio10) {
        let _ = backlight10.set_high();
        println!("   [OK] Пин GPIO 10 переведен в HIGH");
    }

    // Проверяем GPIO 44 (часто используется в обновленных ревизиях 7" / 7B)
    if let Ok(mut backlight44) = esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio44) {
        let _ = backlight44.set_high();
        println!("   [OK] Пин GPIO 44 переведен в HIGH");
    }

    // Проверяем GPIO 45 (альтернативный пин на некоторых китайских HMI-панелях)
    if let Ok(mut backlight45) = esp_idf_hal::gpio::PinDriver::output(peripherals.pins.gpio45) {
        let _ = backlight45.set_high();
        println!("   [OK] Пин GPIO 45 переведен в HIGH");
    }

    // Запуск контроллера дисплея
    println!("3. Вызов функции init_waveshare_rgb_panel()...");
    let panel_handle = init_waveshare_rgb_panel();
    println!("   [OK] Аппаратный дескриптор получен без падения памяти!");

    println!("4. Получение ссылки на фреймбуфер из PSRAM от Си-драйвера...");
    let mut fb_ptr: *mut std::ffi::c_void = std::ptr::null_mut();

    const TOTAL_PIXELS: usize = WIDTH * HEIGHT; // 384000

    unsafe {
        let ret = idf::esp_lcd_rgb_panel_get_frame_buffer(panel_handle, 1, &mut fb_ptr);
        if ret != idf::ESP_OK || fb_ptr.is_null() {
            panic!(
                "❌ Не удалось получить доступ к выделенному буферу PSRAM. Код: {}",
                ret
            );
        }
    }

    // 🔥 ИСПРАВЛЕНИЕ ТИПОВ ДЛЯ EMBEDDED-GRAPHICS-FRAMEBUF:
    // Приводим сырой указатель к изменяемой ссылке на массив ФИКСИРОВАННОЙ длины [Rgb565; 384000].
    // Это исключает ошибку несоответствия трейта FrameBufferBackend!
    let fixed_pixel_array: &mut [Rgb565; TOTAL_PIXELS] =
        unsafe { &mut *(fb_ptr as *mut [Rgb565; TOTAL_PIXELS]) };

    // Передаем ссылку на фиксированный массив в конструктор фреймбуфера
    let mut fbuf = FrameBuf::new(fixed_pixel_array, WIDTH, HEIGHT);

    // Окрашиваем буфер в темно-синий цвет
    let my_background_color = Rgb565::new(2, 4, 12);
    fbuf.clear(my_background_color).unwrap();
    println!("   [OK] Память экрана успешно связана и заполнена цветом.");

    println!("5. Вход в бесконечный цикл трансляции кадров по DMA...");

    let mut frame_counter = 0;
    loop {
        unsafe {
            // Передаем Си-указатель на прорисовку кадра
            idf::esp_lcd_panel_draw_bitmap(panel_handle, 0, 0, WIDTH as i32, HEIGHT as i32, fb_ptr);
        }

        frame_counter += 1;
        if frame_counter % 60 == 0 {
            println!("   [INFO] Отправлено еще 60 кадров на экран (Плата работает стабильно)");
        }

        // Синхронизация частоты под ~60 Гц
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

/// Функция конфигурации таймингов матрицы и GPIO-пинов под спецификацию ESP-IDF v5.2.2
fn init_waveshare_rgb_panel() -> idf::esp_lcd_panel_handle_t {
    //Timings
    let mut timings =
        unsafe { std::mem::MaybeUninit::<idf::esp_lcd_rgb_timing_t>::zeroed().assume_init() };
    timings.pclk_hz = 12_000_000; // Стабильные 12 МГц
    timings.h_res = WIDTH as u32;
    timings.v_res = HEIGHT as u32;
    timings.hsync_back_porch = 40; //+
    timings.hsync_front_porch = 40; //+
    timings.hsync_pulse_width = 48; //+
    timings.vsync_back_porch = 32; //+
    timings.vsync_front_porch = 13; //+
    timings.vsync_pulse_width = 3; //+
    unsafe {
        let mut timing_flags =
            std::mem::MaybeUninit::<idf::esp_lcd_rgb_timing_t__bindgen_ty_1>::zeroed()
                .assume_init();
        timing_flags._bitfield_1 =
            idf::esp_lcd_rgb_timing_t__bindgen_ty_1::new_bitfield_1(0, 0, 0, 1, 0);
        timings.flags = timing_flags;
    }

    //PanelConfig
    let mut rgb_panel_config =
        unsafe { std::mem::MaybeUninit::<idf::esp_lcd_rgb_panel_config_t>::zeroed().assume_init() };
    // Базовая конфигурация полей структуры v5.2.2
    rgb_panel_config.clk_src = idf::soc_periph_lcd_clk_src_t_LCD_CLK_SRC_DEFAULT; //+
    rgb_panel_config.timings = timings;
    rgb_panel_config.data_width = 16;
    rgb_panel_config.bits_per_pixel = 16;
    rgb_panel_config.bounce_buffer_size_px = 0;
    rgb_panel_config.sram_trans_align = 32;
    rgb_panel_config.psram_trans_align = 64;
    // rgb_panel_config.hsync_gpio_num = 39; // HSYNC
    rgb_panel_config.hsync_gpio_num = 46; // HSYNC_from_arduino

    // rgb_panel_config.vsync_gpio_num = 40; // VSYNC
    rgb_panel_config.vsync_gpio_num = 3; // VSYNC_from_arduino

    // rgb_panel_config.pclk_gpio_num = 41; // PCLK
    rgb_panel_config.pclk_gpio_num = 7; // PCLK_from_arduino

    // rgb_panel_config.de_gpio_num = 42; // DE
    rgb_panel_config.de_gpio_num = 5; //DE from arduino

    rgb_panel_config.disp_gpio_num = -1;

    // В ESP-IDF v5.2.x количество буферов задается в поле num_fbs
    rgb_panel_config.num_fbs = 1;
    // Фабричная карта пинов данных Waveshare 7"
    // rgb_panel_config.data_gpio_nums = [15, 7, 6, 5, 4, 9, 46, 3, 8, 18, 17, 14, 21, 47, 48, 45];

    // from_arduino: D0-D15 занимают пины: GPIO 14, 38, 18, 17, 10, 39, 0, 45, 48, 47, 21, 1, 2, 42, 41, 40.
    rgb_panel_config.data_gpio_nums = [14, 38, 18, 17, 10, 39, 0, 45, 48, 47, 21, 1, 2, 42, 41, 40];
    unsafe {
        let mut panel_flags =
            std::mem::MaybeUninit::<idf::esp_lcd_rgb_panel_config_t__bindgen_ty_1>::zeroed()
                .assume_init();

        panel_flags._bitfield_1 = idf::esp_lcd_rgb_panel_config_t__bindgen_ty_1::new_bitfield_1(
            0, // disp_active_low = 0
            0, // refresh_on_demand = 0
            1, // fb_in_psram = 1 (Включает выделение буфера во внешней оперативной памяти PSRAM!)
            0, // big_endian = 0
            0, // reserved = 0
            0,
        );
        rgb_panel_config.flags = panel_flags;
    }

    //Panel
    let mut panel_handle: idf::esp_lcd_panel_handle_t = std::ptr::null_mut();
    unsafe {
        let ret = idf::esp_lcd_new_rgb_panel(&rgb_panel_config, &mut panel_handle);
        if ret != idf::ESP_OK {
            panic!("❌ Ошибка создания RGB панели: код {}", ret);
        }

        let ret = idf::esp_lcd_panel_init(panel_handle);
        if ret != idf::ESP_OK {
            panic!("❌ Ошибка инициализации RGB панели: код {}", ret);
        }
    }
    println!("✅ [Driver]: Аппаратный RGB-контроллер успешно запущен во внешней памяти!");
    panel_handle
}

use embedded_graphics::{
    Pixel,
    pixelcolor::Rgb565,
    prelude::{DrawTarget, OriginDimensions, RgbColor, Size},
};
// use embedded_gui::prelude::Rgb565;
use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_sys::{
    self as idf, EspError, esp_lcd_panel_handle_t, esp_lcd_rgb_panel_config_t, esp_lcd_rgb_timing_t,
};
use slint::platform::software_renderer::{LineBufferProvider, Rgb565Pixel};

#[derive(Default)]
pub struct Display {
    panel_handle: esp_lcd_panel_handle_t,
    panel_config: esp_lcd_rgb_panel_config_t,
    // pub frame_buffer: &'static mut [slint::platform::software_renderer::Rgb565Pixel],
    pub frame_buffer: &'static mut [Rgb565Pixel],
    // pub frame_buffer: Vec<Rgb565Pixel>,
    // pub frame_buffer: &mut[],
}
const WIDTH: usize = 800;
const HEIGHT: usize = 480;
impl Display {
    const DEFAULT_HZ: u32 = 14_000_000;
    const WIDTH: usize = 800;
    const HEIGHT: usize = 480;
    pub fn default() -> Self {
        Display::new(Self::WIDTH, Self::HEIGHT, Self::DEFAULT_HZ)
    }
    pub fn new(width: usize, heigth: usize, hz: u32) -> Self {
        let mut display = Display {
            panel_handle: std::ptr::null_mut(),
            panel_config: unsafe {
                std::mem::MaybeUninit::<idf::esp_lcd_rgb_panel_config_t>::zeroed().assume_init()
            },
            ..Default::default()
        };
        let timings = display.init_timings(width, heigth, hz);
        display.init_panel_config(timings);
        display.init_panel_handle();
        // display.frame_buffer = vec![Rgb565Pixel(0); 800 * 480];
        display.frame_buffer = unsafe {
            let mut fb_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let ret = idf::esp_lcd_rgb_panel_get_frame_buffer(display.panel_handle, 1, &mut fb_ptr);
            if ret != idf::ESP_OK || fb_ptr.is_null() {
                println!("❌ Не удалось получить указатель на фреймбуфер в PSRAM!");
            }
            let frame_buffer = std::slice::from_raw_parts_mut(
                fb_ptr as *mut Rgb565Pixel, // Кастуем сразу в тип пикселей Slint
                Self::WIDTH * Self::HEIGHT,
            );
            frame_buffer
        };
        display.init_i2c();
        display
    }
    fn init_i2c(&self) {
        // CH422G
        const ADDRESS: u8 = 0x24;
        let p = Peripherals::take();
        if let Err(err) = p {
            print!("[i2c_err]{}", err.to_string());
        }
        let p = p.unwrap();
        let i2c_config = I2cConfig::new().baudrate(Hertz(100_000));
        let mut i2c = I2cDriver::new(
            p.i2c0,
            p.pins.gpio8, // SDA
            p.pins.gpio9, // SCL
            &i2c_config,
        )
        .unwrap();
        println!("⚙️ Включение подсветки и сброс матрицы через I2C...");
        print!("[Отправка байта на 7ми битный адреса] ");
        match i2c.write(ADDRESS, &[0x1C], 1000) {
            Ok(_) => println!("✅"),
            Err(err) => println!("❎: {}", err.to_string()),
        };
        FreeRtos::delay_ms(50);

        print!("[Подача питания и включение подсветки] ");
        match i2c.write(ADDRESS, &[0x1E], 1000) {
            Ok(_) => println!("✅"),
            Err(err) => println!("❎: {}", err.to_string()),
        }
        FreeRtos::delay_ms(100);
    }
    fn init_timings(&self, width: usize, height: usize, hz: u32) -> idf::esp_lcd_rgb_timing_t {
        //Timings
        let mut timings =
            unsafe { std::mem::MaybeUninit::<idf::esp_lcd_rgb_timing_t>::zeroed().assume_init() };
        timings.pclk_hz = hz;
        timings.h_res = width as u32;
        timings.hsync_pulse_width = 4; //48; //+
        timings.hsync_back_porch = 8; //+
        timings.hsync_front_porch = 8; //40; //+

        timings.v_res = height as u32;
        timings.vsync_pulse_width = 4; //3; //+
        timings.vsync_back_porch = 8; //32; //+
        timings.vsync_front_porch = 8; //13; //+

        let mut timing_flags = unsafe {
            std::mem::MaybeUninit::<idf::esp_lcd_rgb_timing_t__bindgen_ty_1>::zeroed().assume_init()
        };
        timing_flags._bitfield_1 = idf::esp_lcd_rgb_timing_t__bindgen_ty_1::new_bitfield_1(
            0, // hsync_idle_low = 0
            0, // vsync_idle_low = 0
            0, // de_idle_high = 0
            1, // pclk_active_neg = 1 (Данные фиксируются по спаду импульса)
            0, // reserved = 0
        );
        timings.flags = timing_flags;
        timings
    }
    fn init_panel_config(&mut self, timings: esp_lcd_rgb_timing_t) {
        //PanelConfig
        self.panel_config.clk_src = idf::soc_periph_lcd_clk_src_t_LCD_CLK_SRC_DEFAULT; //+
        self.panel_config.timings = timings;
        self.panel_config.data_width = 16;
        self.panel_config.bits_per_pixel = 16;
        self.panel_config.bounce_buffer_size_px = 800 * 10;
        self.panel_config.sram_trans_align = 32;
        self.panel_config.psram_trans_align = 64;

        self.panel_config.hsync_gpio_num = 46; // HSYNC_from_arduino
        self.panel_config.vsync_gpio_num = 3; // VSYNC_from_arduino
        self.panel_config.pclk_gpio_num = 7; // PCLK_from_arduino
        self.panel_config.de_gpio_num = 5; //DE from arduino
        self.panel_config.disp_gpio_num = -1;

        self.panel_config.num_fbs = 1;
        let gpio_array = [
            14, // B0
            38, // B1
            18, // B2
            17, // B3
            10, // B4
            39, // G0
            0,  // G1
            45, // G2
            48, // G3
            47, // G4
            21, // G5
            1,  // R0
            2,  // R1
            42, // R2
            41, // R3
            40, // R4
        ];
        self.panel_config.data_gpio_nums = gpio_array;
        let mut panel_flags = unsafe {
            std::mem::MaybeUninit::<idf::esp_lcd_rgb_panel_config_t__bindgen_ty_1>::zeroed()
                .assume_init()
        };
        panel_flags._bitfield_1 = idf::esp_lcd_rgb_panel_config_t__bindgen_ty_1::new_bitfield_1(
            0, // disp_active_low = 0
            0, // refresh_on_demand = 0
            1, // fb_in_psram = 1 (Включает выделение буфера во внешней оперативной памяти PSRAM!)
            0, // big_endian = 0
            0, // reserved = 0
            0,
        );
        self.panel_config.flags = panel_flags;
    }
    fn init_panel_handle(&mut self) {
        unsafe {
            print!("[Cоздание новой panel_handle] ");
            match idf::esp_lcd_new_rgb_panel(&self.panel_config, &mut self.panel_handle) {
                idf::ESP_OK => println!("✅"),
                err => println!("❎ : {}", EspError::from(err).unwrap()),
            }
            print!("[Инициализация panel_handle] ");
            match idf::esp_lcd_panel_init(self.panel_handle) {
                idf::ESP_OK => println!("✅"),
                err => println!("❎ : {}", EspError::from(err).unwrap()),
            }
        }
    }
    pub fn panel_handle(&mut self) -> esp_lcd_panel_handle_t {
        self.panel_handle
    }
    pub fn clear_screen(&mut self, color: Rgb565) {
        let raw_color = ((color.r() as u16) << 11) | ((color.g() as u16) << 5) | (color.b() as u16);
        let shift_x: i32 = 32;
        self.frame_buffer.fill(Rgb565Pixel(raw_color));
        if shift_x != 0 {
            for y in 0..Self::HEIGHT {
                let row_start = y * Self::WIDTH;
                //
                if shift_x > 0 {
                    // Сдвигаем строку вправо
                    let shift = shift_x as usize;
                    if shift < Self::WIDTH {
                        self.frame_buffer[(row_start + shift)..(row_start + Self::WIDTH)]
                            .fill(Rgb565Pixel(raw_color));
                    }
                } else {
                    // Сдвигаем строку влево
                    let shift = (-shift_x) as usize;
                    if shift < Self::WIDTH {
                        self.frame_buffer[(row_start + shift)..(row_start + Self::WIDTH)]
                            .fill(Rgb565Pixel(raw_color));
                    }
                }
            }
        }
    }
}

impl slint::platform::software_renderer::LineBufferProvider for Display {
    type TargetPixel = Rgb565Pixel;
    fn process_line(
        &mut self,
        line: usize,                    // Текущая строка Y (от 0 до 477)
        range: core::ops::Range<usize>, // Диапазон пикселей по X (например, 0..800)
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        // 1. ЗАЩИТА: Если Slint передал пустую строку, ничего не делаем
        if range.is_empty() {
            return;
        }

        // 2. Вычисляем начальный индекс строки в одномерном фреймбуфере (Y * WIDTH)
        let row_start_index = line * Self::WIDTH;

        // 3. Выделяем срез (slice) во фреймбуфере, куда Slint должен нарисовать пиксели
        // Используем range, чтобы Slint писал строго в свои координаты по X
        let buffer_chunk =
            &mut self.frame_buffer[row_start_index + range.start..row_start_index + range.end];

        // 4. Запускаем рендеринг Slint прямо в память PSRAM (без промежуточных копирований!)
        render_fn(buffer_chunk);

        // 5. Коррекция смещения (shift_x), если экран аппаратно сдвинут.
        // Если ваш экран показывает картинку ровно, этот блок (строки ниже) можно удалить.
        let shift_x: i32 = 32; // Ваше смещение из метода clear_screen
        if shift_x != 0 && line < Self::HEIGHT {
            // Здесь при необходимости можно сделать сдвиг памяти через .copy_within()
        }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(Self::WIDTH as u32, Self::HEIGHT as u32)
    }
}

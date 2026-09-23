use std::{
    cell::RefCell,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{OriginDimensions, RgbColor, Size},
};
use esp_idf_hal::{delay::FreeRtos, i2c::I2cDriver};
use esp_idf_sys::{
    self as idf, EspError, esp_lcd_panel_handle_t, esp_lcd_rgb_panel_config_t, esp_lcd_rgb_timing_t,
};
use slint::{
    LogicalPosition, PhysicalSize,
    platform::{
        Platform, PointerEventButton, WindowEvent, set_platform,
        software_renderer::{
            LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel,
        },
        update_timers_and_animations,
    },
};
#[derive(Clone)]
pub struct Display {
    panel_handle: esp_lcd_panel_handle_t,
    panel_config: esp_lcd_rgb_panel_config_t,
    frame_buffer: Rc<RefCell<&'static mut [Rgb565Pixel]>>,
    i2c: Rc<RefCell<I2cDriver<'static>>>,
    platform: DisplayPlatform,
}

impl Display {
    const DEFAULT_HZ: u32 = 14_000_000;
    const WIDTH: usize = 800;
    const HEIGHT: usize = 480;
    pub fn default(i2c: Rc<RefCell<I2cDriver<'static>>>) -> Self {
        Display::new(Self::WIDTH, Display::HEIGHT, Self::DEFAULT_HZ, i2c)
    }
    pub fn new(width: usize, heigth: usize, hz: u32, i2c: Rc<RefCell<I2cDriver<'static>>>) -> Self {
        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        set_platform(Box::new(DisplayPlatform::new(
            window.clone(),
            Instant::now(),
        )))
        .expect("Ну установилась платформа");
        let mut display = Display {
            panel_handle: std::ptr::null_mut(),
            panel_config: unsafe {
                std::mem::MaybeUninit::<idf::esp_lcd_rgb_panel_config_t>::zeroed().assume_init()
            },
            i2c: i2c,
            frame_buffer: Rc::new(RefCell::new(&mut [])),
            platform: DisplayPlatform::new(window, Instant::now()),
        };

        let timings = display.init_timings(width, heigth, hz);
        display.init_panel_config(timings);
        display.init_panel_handle();
        display.init_buffer();
        display.init_i2c();
        display
    }
    fn init_buffer(&mut self) {
        *self.frame_buffer.borrow_mut() = unsafe {
            let mut fb_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let ret = idf::esp_lcd_rgb_panel_get_frame_buffer(self.panel_handle, 1, &mut fb_ptr);
            if ret != idf::ESP_OK || fb_ptr.is_null() {
                println!("❌ Не удалось получить указатель на фреймбуфер в PSRAM!");
            }
            let frame_buffer = std::slice::from_raw_parts_mut(
                fb_ptr as *mut Rgb565Pixel,
                Self::WIDTH * Self::HEIGHT,
            );
            frame_buffer
        };
    }
    fn init_i2c(&mut self) {
        // CH422G
        const ADDRESS: u8 = 0x24;
        println!("⚙️ Включение подсветки и сброс матрицы через I2C...");
        print!("[Отправка байта на 7ми битный адреса] ");
        match self.i2c.borrow_mut().write(ADDRESS, &[0x1C], 1000) {
            Ok(_) => println!("✅"),
            Err(err) => println!("❎: {}", err.to_string()),
        };
        FreeRtos::delay_ms(50);

        print!("[Подача питания и включение подсветки] ");
        match self.i2c.borrow_mut().write(ADDRESS, &[0x1E], 1000) {
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
        timings.hsync_pulse_width = 4;
        timings.hsync_back_porch = 8;
        timings.hsync_front_porch = 8;

        timings.v_res = height as u32;
        timings.vsync_pulse_width = 4;
        timings.vsync_back_porch = 8;
        timings.vsync_front_porch = 8;

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
        self.panel_config.bounce_buffer_size_px = 800 * 3;
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
    // pub fn panel_handle(&mut self) -> esp_lcd_panel_handle_t {
    //     self.panel_handle
    // }
    pub fn clear_screen(&mut self, color: Rgb565) {
        let raw_color = ((color.r() as u16) << 11) | ((color.g() as u16) << 5) | (color.b() as u16);
        let shift_x: i32 = 32;
        self.frame_buffer.borrow_mut().fill(Rgb565Pixel(raw_color));
        if shift_x != 0 {
            for y in 0..Self::HEIGHT {
                let row_start = y * Self::WIDTH;
                //
                if shift_x > 0 {
                    // Сдвигаем строку вправо
                    let shift = shift_x as usize;
                    if shift < Self::WIDTH {
                        self.frame_buffer.borrow_mut()
                            [(row_start + shift)..(row_start + Self::WIDTH)]
                            .fill(Rgb565Pixel(raw_color));
                    }
                } else {
                    // Сдвигаем строку влево
                    let shift = (-shift_x) as usize;
                    if shift < Self::WIDTH {
                        self.frame_buffer.borrow_mut()
                            [(row_start + shift)..(row_start + Self::WIDTH)]
                            .fill(Rgb565Pixel(raw_color));
                    }
                }
            }
        }
    }
    pub fn touch_pad(&self) -> GT911 {
        GT911::new(self.i2c.clone())
    }
    fn window(&self) -> Rc<MinimalSoftwareWindow> {
        self.platform.window.clone()
    }
    pub fn i2c(&self) -> Rc<RefCell<I2cDriver<'static>>> {
        Rc::clone(&self.i2c)
    }
    pub fn update(&self) {
        self.platform.window.draw_if_needed(|render| {
            render.render_by_line(self);
        });
    }
    pub fn handle_touch_pad(
        &self,
        mut was_pressed: bool,
        mut release_debounce_counter: i32,
        mut last_slint_pos: LogicalPosition,
    ) {
        let window = self.window();
        update_timers_and_animations();
        if let Some((x, y)) = self.touch_pad().read_touch() {
            release_debounce_counter = 0;
            last_slint_pos = slint::LogicalPosition::new(x as f32, y as f32);

            window.dispatch_event(slint::platform::WindowEvent::PointerMoved {
                position: last_slint_pos,
            });
            if !was_pressed {
                window.dispatch_event(WindowEvent::PointerPressed {
                    position: last_slint_pos,
                    button: PointerEventButton::Left,
                });
                was_pressed = true;
            }
        } else {
            if was_pressed {
                release_debounce_counter += 1;
                if release_debounce_counter >= 3 {
                    window.dispatch_event(WindowEvent::PointerReleased {
                        position: last_slint_pos,
                        button: PointerEventButton::Left,
                    });
                    was_pressed = false;
                    release_debounce_counter = 0;
                }
            }
        }
        self.update();
        FreeRtos::delay_ms(8);
    }
}

pub struct GT911 {
    driver: Rc<RefCell<I2cDriver<'static>>>,
}
impl GT911 {
    fn new(i2c: Rc<RefCell<I2cDriver<'static>>>) -> Self {
        Self { driver: i2c }
    }
    pub fn read_touch(&mut self) -> Option<(u16, u16)> {
        const GT911_ADDRESS: u8 = 0x14;
        const STATUS_ADDRESS: u16 = 0x814E;
        let mut status_reg = [0u8; 1];
        if let Err(_) = self.i2c_read_reg(GT911_ADDRESS, STATUS_ADDRESS, &mut status_reg) {
            return None;
        }

        let status = status_reg[0];
        let buffer_status = status & 0x80; // Проверяем флаг готовности данных
        let number_of_touches = status & 0x0F; // Количество пальцев (от 0 до 5)

        if buffer_status == 0 || number_of_touches == 0 {
            // Если новых нажатий нет, обязательно сбрасываем буфер (пишем 0 в статус), чтобы чип не завис
            let _ = self.i2c_write_reg(GT911_ADDRESS, STATUS_ADDRESS, 0);
            return None;
        }

        // 2. Считываем координаты первой точки касания (Регистры 0x8150 - 0x8153)
        let mut coord_buf = [0u8; 4];
        if let Ok(_) = self.i2c_read_reg(GT911_ADDRESS, 0x8150, &mut coord_buf) {
            let x = u16::from_le_bytes([coord_buf[0], coord_buf[1]]);
            let y = u16::from_le_bytes([coord_buf[2], coord_buf[3]]);
            let _ = self.i2c_write_reg(GT911_ADDRESS, 0x814E, 0);
            return Some((x, y));
        }
        // Если новых нажатий нет, обязательно сбрасываем буфер (пишем 0 в статус), чтобы чип не завис
        let _ = self.i2c_write_reg(GT911_ADDRESS, STATUS_ADDRESS, 0);
        None
    }
    fn i2c_read_reg(&mut self, addr: u8, reg: u16, buf: &mut [u8]) -> Result<(), EspError> {
        let reg_bytes = reg.to_be_bytes();
        self.driver
            .borrow_mut()
            .write_read(addr, &reg_bytes, buf, 100)
    }
    fn i2c_write_reg(&mut self, addr: u8, reg: u16, value: u8) -> Result<(), EspError> {
        let [b1, b2] = reg.to_be_bytes();
        let payload = [b1, b2, value];
        self.driver.borrow_mut().write(addr, &payload, 100)
    }
}

impl LineBufferProvider for &Display {
    type TargetPixel = Rgb565Pixel;
    fn process_line(
        &mut self,
        line: usize,         // Текущая строка Y (от 0 до 477)
        range: Range<usize>, // Диапазон пикселей по X (например, 0..800)
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let offset = line * Display::WIDTH;
        let line_slice = &mut self.frame_buffer.borrow_mut()[offset..offset + Display::WIDTH];
        render_fn(&mut line_slice[range]);
    }
}
impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(Self::WIDTH as u32, Self::HEIGHT as u32)
    }
}

#[derive(Clone)]
pub struct DisplayPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: Instant,
}

impl DisplayPlatform {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 480;
    fn new(window: Rc<MinimalSoftwareWindow>, start_time: Instant) -> Self {
        window.set_size(PhysicalSize::new(Self::WIDTH as u32, Self::HEIGHT as u32));
        Self { window, start_time }
    }

    pub fn window_mut(&mut self) -> &mut Rc<MinimalSoftwareWindow> {
        &mut self.window
    }
}

impl Platform for DisplayPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.start_time.elapsed()
    }
}

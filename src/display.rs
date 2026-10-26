use std::{
    cell::RefCell,
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

use embedded_graphics::prelude::{OriginDimensions, Size};
use esp_idf_hal::{delay::FreeRtos, i2c::I2cDriver};
use esp_idf_sys::{
    self as idf, EspError, esp_lcd_panel_handle_t, esp_lcd_rgb_panel_config_t, esp_lcd_rgb_timing_t,
};
use log::{error, info};
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
    fb0: *mut u16,
    fb1: *mut u16,
    active_fb: usize,
    i2c: Rc<RefCell<I2cDriver<'static>>>,
    platform: DisplayPlatform,
    status: TouchPadStatus,
}
unsafe impl Send for Display {}
#[derive(Clone, Debug)]
struct TouchPadStatus {
    touched: bool,
    release_counter: u8,
    last_position: LogicalPosition,
}
#[derive(Clone)]
pub struct DisplayPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: Instant,
}
impl Display {
    const DEFAULT_HZ: u32 = 13_000_000;
    const WIDTH: usize = 800;
    const HEIGHT: usize = 480;
    pub fn default(i2c: Rc<RefCell<I2cDriver<'static>>>) -> Self {
        Display::new(Self::WIDTH, Display::HEIGHT, Self::DEFAULT_HZ, i2c)
    }
    pub fn new(width: usize, heigth: usize, hz: u32, i2c: Rc<RefCell<I2cDriver<'static>>>) -> Self {
        let window = MinimalSoftwareWindow::new(RepaintBufferType::SwappedBuffers);
        if let Err(err) = set_platform(Box::new(DisplayPlatform::new(
            window.clone(),
            Instant::now(),
        ))) {
            error!("Не установилась платформа: {}", err);
        }
        let mut display = Display {
            panel_handle: std::ptr::null_mut(),
            panel_config: unsafe {
                std::mem::MaybeUninit::<idf::esp_lcd_rgb_panel_config_t>::zeroed().assume_init()
            },
            i2c: i2c,
            fb0: std::ptr::null_mut(),
            fb1: std::ptr::null_mut(),
            active_fb: 0,
            platform: DisplayPlatform::new(window, Instant::now()),
            status: {
                TouchPadStatus {
                    touched: false,
                    release_counter: 0,
                    last_position: LogicalPosition::new(0.0, 0.0),
                }
            },
        };

        display.init_i2c();
        let timings = display.init_timings(width, heigth, hz);
        display
            .init_panel_config(timings)
            .init_panel_handle()
            .init_buffer()
    }
    //public
    pub fn touch_pad(&self) -> GT911 {
        GT911::new(self.i2c.clone())
    }
    pub fn i2c(&self) -> Rc<RefCell<I2cDriver<'static>>> {
        Rc::clone(&self.i2c)
    }
    pub fn update(&mut self) {
        let was_redraw = self.platform.window.draw_if_needed(|render| {
            render.render_by_line(&mut self.clone());
        });
        if was_redraw {
            self.swap_buffers();
        }
    }
    pub fn handle_touch_pad(&mut self) {
        let window = self.platform.window.clone();
        update_timers_and_animations();
        if let Some((x, y)) = self.touch_pad().read_touch() {
            self.status.release_counter = 0;
            self.status.last_position = slint::LogicalPosition::new(x as f32, y as f32);

            window.dispatch_event(slint::platform::WindowEvent::PointerMoved {
                position: self.status.last_position,
            });
            if !self.status.touched {
                window.dispatch_event(WindowEvent::PointerPressed {
                    position: self.status.last_position,
                    button: PointerEventButton::Left,
                });
                self.status.touched = true;
            }
        } else {
            if self.status.touched {
                self.status.release_counter += 1;
                if self.status.release_counter >= 3 {
                    window.dispatch_event(WindowEvent::PointerReleased {
                        position: self.status.last_position,
                        button: PointerEventButton::Left,
                    });
                    self.status.touched = false;
                    self.status.release_counter = 0;
                }
            }
        }
        self.update();
        FreeRtos::delay_ms(8);
    }
    //private
    fn init_buffer(mut self) -> Self {
        unsafe {
            let mut fb0: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut fb1: *mut std::ffi::c_void = std::ptr::null_mut();
            let ret =
                idf::esp_lcd_rgb_panel_get_frame_buffer(self.panel_handle, 2, &mut fb0, &mut fb1);
            if ret != idf::ESP_OK || fb0.is_null() || fb1.is_null() {
                error!("❌ Не удалось получить фреймбуферы! {:?}", ret);
            }
            self.fb0 = fb0 as *mut u16;
            self.fb1 = fb1 as *mut u16;
        }
        self
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
        timings.hsync_pulse_width = 1;
        timings.hsync_back_porch = 36;
        timings.hsync_front_porch = 20;

        timings.v_res = height as u32;
        timings.vsync_pulse_width = 1;
        timings.vsync_back_porch = 6;
        timings.vsync_front_porch = 4;

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
    fn init_panel_config(mut self, timings: esp_lcd_rgb_timing_t) -> Self {
        //PanelConfig
        self.panel_config.clk_src = idf::soc_periph_lcd_clk_src_t_LCD_CLK_SRC_DEFAULT; //+
        self.panel_config.timings = timings;
        self.panel_config.data_width = 16;
        self.panel_config.bits_per_pixel = 16;
        self.panel_config.bounce_buffer_size_px = Self::WIDTH * 10;
        self.panel_config.sram_trans_align = 32;
        self.panel_config.psram_trans_align = 64;

        self.panel_config.hsync_gpio_num = 46; // HSYNC_from_arduino
        self.panel_config.vsync_gpio_num = 3; // VSYNC_from_arduino
        self.panel_config.pclk_gpio_num = 7; // PCLK_from_arduino
        self.panel_config.de_gpio_num = 5; //DE from arduino
        self.panel_config.disp_gpio_num = -1;

        self.panel_config.num_fbs = 2;
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
        self
    }
    fn init_panel_handle(mut self) -> Self {
        unsafe {
            info!("[Cоздание новой panel_handle] ");
            match idf::esp_lcd_new_rgb_panel(&self.panel_config, &mut self.panel_handle) {
                idf::ESP_OK => info!("✅"),
                err => error!("❎ : {}", EspError::from(err).unwrap()),
            }
            info!("[Инициализация panel_handle] ");
            match idf::esp_lcd_panel_init(self.panel_handle) {
                idf::ESP_OK => info!("✅"),
                err => error!("❎ : {}", EspError::from(err).unwrap()),
            }
        }
        self
    }
    fn actual_buffer(&self) -> *mut u16 {
        if self.active_fb == 0 {
            self.fb1
        } else {
            self.fb0
        }
    }
    fn swap_buffers(&mut self) {
        let draw_buf = self.actual_buffer();
        unsafe {
            idf::esp_lcd_panel_draw_bitmap(
                self.panel_handle,
                0,
                0,
                Self::WIDTH as i32,
                Self::WIDTH as i32,
                draw_buf as *const std::ffi::c_void,
            );
        }
        self.active_fb = !self.active_fb;
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
impl LineBufferProvider for &mut Display {
    type TargetPixel = Rgb565Pixel;
    fn process_line(
        &mut self,
        line: usize,         // Текущая строка Y (от 0 до 477)
        range: Range<usize>, // Диапазон пикселей по X (например, 0..800)
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let offset = line * Display::WIDTH;
        let draw_ptr = self.actual_buffer();

        let fb = unsafe {
            std::slice::from_raw_parts_mut(
                draw_ptr as *mut Rgb565Pixel,
                Display::WIDTH * Display::HEIGHT,
            )
        };
        let line_slice = &mut fb[offset..offset + Display::WIDTH];
        render_fn(&mut line_slice[range]);
    }
}
impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(Self::WIDTH as u32, Self::HEIGHT as u32)
    }
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

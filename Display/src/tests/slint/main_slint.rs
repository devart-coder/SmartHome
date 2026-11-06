use std::cell::RefCell;
use std::time::{Duration, Instant};
use std::{rc::Rc, sync::Arc};

use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::{DrawTarget, Size};
use embedded_graphics::primitives::Rectangle;
// Добавьте эту строчку
use embedded_graphics::{Pixel, pixelcolor::Rgb565, prelude::Point};
use esp_idf_hal::delay::FreeRtos;
use lcd_touch_display::display::{self};
use slint::platform::software_renderer::{LineBufferProvider, MinimalSoftwareWindow, Rgb565Pixel};

slint::include_modules!();

struct SlintDisplayBridge<'a> {
    display: &'a mut lcd_touch_display::display::Display,
}

impl slint::platform::software_renderer::LineBufferProvider for SlintDisplayBridge<'_> {
    type TargetPixel = slint::platform::software_renderer::Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        if range.is_empty() {
            return;
        }

        // Задаем ширину экрана жестко (800)
        const SCREEN_WIDTH: usize = 800;
        let row_start = line * SCREEN_WIDTH;

        // Выделяем срез памяти во фреймбуфере вашего дисплея, который лежит в PSRAM
        let buffer_chunk =
            &mut self.display.frame_buffer[row_start + range.start..row_start + range.end];

        // Slint рисует прямо в PSRAM
        render_fn(buffer_chunk);
    }
}

struct MyDisplayPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: Instant,
}

impl slint::platform::Platform for MyDisplayPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        self.start_time.elapsed()
    }
}
fn main() {
    // Обязательная системная инициализация esp-idf
    esp_idf_sys::link_patches();
    let display = Rc::new(RefCell::new(display::Display::default()));
    // display.clear_screen(Rgb565::new(19, 15, 19));
    // Инициализация графического движка Slint
    let window = MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::NewBuffer,
    );
    slint::platform::set_platform(Box::new(MyDisplayPlatform {
        window: window.clone(),
        start_time: Instant::now(),
    }))
    .expect("Ошибка платформы");

    // 2. СОЗДАЕМ ОКНО ИЗ ДЕКЛАРАЦИИ
    let main_window = MainWindow::new().unwrap();
    main_window.show().unwrap();
    const DISPLAY_WIDTH: usize = 800;
    const DISPLAY_HEIGHT: usize = 480;
    window.set_size(slint::PhysicalSize::new(
        DISPLAY_WIDTH as u32,
        DISPLAY_HEIGHT as u32,
    ));
    println!("Декларативный UI успешно запущен!");
    // let mut line_buffer = [slint::platform::software_renderer::Rgb565Pixel(0); DISPLAY_WIDTH];
    loop {
        // Продвигаем время Slint для анимаций
        slint::platform::update_timers_and_animations();
        let display_clone = display.clone();
        // Запрашиваем отрисовку интерфейса
        window.draw_if_needed(|renderer| {
            let mut display_lock = display_clone.borrow_mut();

            // Создаем мост, передавая туда ссылку на дисплей
            let bridge = SlintDisplayBridge {
                display: &mut *display_lock,
            };

            // Передаем мост ПО ЗНАЧЕНИЮ. Это гарантированно удовлетворяет трейт-баунд Slint!
            renderer.render_by_line(bridge);
        });

        // Даем отдохнуть FreeRTOS, чтобы экран обновлялся плавно
        FreeRtos::delay_ms(16);
    }
}

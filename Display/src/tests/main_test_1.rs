use embedded_graphics::prelude::Primitive;
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, jis_x0201::FONT_10X20},
    prelude::{DrawTarget, Point, RgbColor, Size},
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_gui::prelude::Rgb565;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_sys::{self as idf, EspError};
use lcd_touch_display::display::{self};
fn main() {
    // Обязательная системная инициализация esp-idf
    esp_idf_sys::link_patches();
    let mut display = display::Display::default();
    // let mut display = EspRgbDisplay::new(panel_handle);
    display.clear_screen(Rgb565::new(19, 15, 19));
    // display.clear_screen(Rgb565::new(4, 4, 4));
    let card_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::new(10, 20, 30)) // Синеватый фон
        .stroke_color(Rgb565::WHITE)
        .stroke_width(2)
        .build();

    // Рисуем UI-плашку по центру экрана
    Rectangle::new(Point::new(100, 80), Size::new(600, 320))
        .into_styled(card_style)
        .draw(&mut display)
        .unwrap();

    // Рисуем декоративный круг
    Circle::new(Point::new(120, 100), 40)
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
        .draw(&mut display);

    // Выводим текст поверх UI
    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::new("ESP32-S3 Smart Home UI", Point::new(180, 125), text_style)
        .draw(&mut display)
        .unwrap();

    Text::new(
        "Rust Embedded GUI Running OK!",
        Point::new(180, 160),
        text_style,
    )
    .draw(&mut display)
    .unwrap();

    println!("Графика успешно отрисована!");

    loop {
        FreeRtos::delay_ms(1000);
    }
    // FreeRtos::delay_ms(1000);
    // loop {
    // ВАРИАНТ 1: Глубокий темно-серый (почти черный)
    // В двоичном коде это ровные цепочки нулей, шина отдыхает, мерцание уйдет.
    // display.clear_screen(Rgb565::new(8, 16, 8));
    //
    // FreeRtos::delay_ms(1000);
    // ВАРИАНТ 2: Сбалансированный серый средней яркости
    // Биты распределены симметрично, что гасит наводки на линию тактирования.
    // display.clear_screen(Rgb565::new(16, 32, 16));
    //
    // FreeRtos::delay_ms(1000);
    // ВАРИАНТ 3: Чистый белый цвет (если нужно проверить, мерцает ли матрица вообще)
    // Все линии шины данных зажаты в логическую «1», шума на шлейфе физически нет.
    // display.clear_screen(Rgb565::new(31, 63, 31));
    // FreeRtos::delay_ms(1000);
    // }
}

#![no_std]
#![no_main]

use embedded_graphics::image::{Image, ImageRawLE};
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_6X13_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
#[allow(unused)]
use esp_backtrace as _;
use esp_hal::gpio::Io;
use esp_hal::i2c::I2C;
use esp_hal::system::SystemControl;
use esp_hal::{clock::ClockControl, delay::Delay, peripherals::Peripherals, prelude::*};
use esp_println::println;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::{DisplayRotation, DisplaySize128x64};
use ssd1306::{I2CDisplayInterface, Ssd1306};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let scl = io.pins.gpio2;
    let sda = io.pins.gpio3;
    let i2c = I2C::new(peripherals.I2C0, sda, scl, 100.kHz(), &clocks, None);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    println!("calling display.init()");

    display.flush().unwrap();

    display.clear(BinaryColor::Off).unwrap();

    //********* display some text

    // creating MonoTextStyleBuilder
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let text_style_bold = MonoTextStyleBuilder::new()
        .font(&FONT_6X13_BOLD)
        .text_color(BinaryColor::On)
        .build();

    println!("displaying Hello world! on LCD");
    Text::with_baseline(
        "Hello Rust World!....",
        Point::zero(),
        text_style_bold,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    println!("displaying Hello Rust! on LCD");
    Text::with_baseline("SSD1306-I2C", Point::new(0, 19), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    // graphics
    delay.delay_millis(2000);
    display.clear(BinaryColor::Off).unwrap();

    Line::new(Point::new(8, 16 + 16), Point::new(8 + 16, 16 + 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(8, 16 + 16), Point::new(8 + 8, 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(8 + 16, 16 + 16), Point::new(8 + 8, 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Rectangle::with_corners(Point::new(48, 16), Point::new(48 + 16, 16 + 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    Circle::new(Point::new(88, 16), 16)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    // pixel square
    delay.delay_millis(2000);
    display.clear(BinaryColor::Off).unwrap();

    // Top side
    display.set_pixel(0, 0, true);
    display.set_pixel(1, 0, true);
    display.set_pixel(2, 0, true);
    display.set_pixel(3, 0, true);

    // Right side
    display.set_pixel(3, 0, true);
    display.set_pixel(3, 1, true);
    display.set_pixel(3, 2, true);
    display.set_pixel(3, 3, true);

    // Bottom side
    display.set_pixel(0, 3, true);
    display.set_pixel(1, 3, true);
    display.set_pixel(2, 3, true);
    display.set_pixel(3, 3, true);

    // Left side
    display.set_pixel(0, 0, true);
    display.set_pixel(0, 1, true);
    display.set_pixel(0, 2, true);
    display.set_pixel(0, 3, true);

    display.flush().unwrap();

    // image
    delay.delay_millis(2000);
    display.clear(BinaryColor::Off).unwrap();
    let im: ImageRawLE<BinaryColor> = ImageRawLE::new(include_bytes!("./rust.raw"), 64);

    Image::new(&im, Point::new(32, 0))
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    let mut count = 0;
    loop {
        println!("count: {count}");
        count += 1;
        delay.delay_millis(1000);
    }
}

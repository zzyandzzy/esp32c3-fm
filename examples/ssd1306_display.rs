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
use esp_backtrace as _;
use esp_hal::i2c::I2C;
use esp_hal::{clock::ClockControl, delay::Delay, peripherals::Peripherals, prelude::*};
use esp_hal::gpio::Io;
use esp_hal::system::SystemControl;
use esp_println::println;
use ssd1306_i2c::{prelude::*, Builder};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let slc = io.pins.gpio8;
    let sda = io.pins.gpio10;
    let i2c = I2C::new(peripherals.I2C0, sda, slc, 100.kHz(), &clocks, None);

    let mut display: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x64NoOffset)
        .with_i2c_addr(0x3c)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_i2c(i2c)
        .into();
    println!("calling display.init()");

    display.init().unwrap();

    display.flush().unwrap();

    display.clear();

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
    display.clear();

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
    display.clear();

    // Top side
    display.set_pixel(0, 0, 1);
    display.set_pixel(1, 0, 1);
    display.set_pixel(2, 0, 1);
    display.set_pixel(3, 0, 1);

    // Right side
    display.set_pixel(3, 0, 1);
    display.set_pixel(3, 1, 1);
    display.set_pixel(3, 2, 1);
    display.set_pixel(3, 3, 1);

    // Bottom side
    display.set_pixel(0, 3, 1);
    display.set_pixel(1, 3, 1);
    display.set_pixel(2, 3, 1);
    display.set_pixel(3, 3, 1);

    // Left side
    display.set_pixel(0, 0, 1);
    display.set_pixel(0, 1, 1);
    display.set_pixel(0, 2, 1);
    display.set_pixel(0, 3, 1);

    display.flush().unwrap();

    // image
    delay.delay_millis(2000);
    display.clear();
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

#![no_std]
#![no_main]
extern crate alloc;

use alloc::format;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{DrawTarget, Point};
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::Drawable;
#[allow(unused)]
use esp_backtrace as _;
use esp_hal::gpio::{GpioPin, Input, Io, Pull};
use esp_hal::i2c::I2C;
use esp_hal::peripherals::I2C0;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::{ErasedTimer, OneShotTimer};
use esp_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl, Blocking,
};
use esp_println::println;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::{DisplayRotation, DisplaySize128x64};
use ssd1306::{I2CDisplayInterface, Ssd1306};
use static_cell::StaticCell;

use esp32c3_fm::event::{key_detection, EventType};

static ONE_SHOT_TIMER: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, (u8, EventType), 64> = Channel::new();

#[embassy_executor::task]
async fn sw1_run(mut sw1_key: Input<'static, GpioPin<7>>) {
    loop {
        sw1_key.wait_for_falling_edge().await;
        key_detection(&sw1_key, move |event_type| {
            println!("event_type:{:?}", event_type);
            CHANNEL.try_send((7, event_type)).ok();
        })
        .await;
    }
}

#[embassy_executor::task]
async fn sw2_run(mut sw2_key: Input<'static, GpioPin<6>>) {
    loop {
        sw2_key.wait_for_falling_edge().await;
        key_detection(&sw2_key, |event_type| {
            println!("event_type:{:?}", event_type);
            CHANNEL.try_send((6, event_type)).ok();
        })
        .await;
    }
}

#[embassy_executor::task]
async fn display_run(i2c: I2C<'static, I2C0, Blocking>) {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    display.flush().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        let msg = CHANNEL.receive().await;
        Text::with_baseline(
            format!("gpio{},{:?}", msg.0, msg.1).as_str(),
            Point::new(0, 19),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();
        display.clear(BinaryColor::Off).unwrap();
    }
}

#[main]
async fn main(spawner: Spawner) {
    alloc();
    esp_println::logger::init_logger_from_env();
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // initialize the timer(s)
    let timer_group = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let one_shot_timer = OneShotTimer::new(timer_group.timer0.into());
    let timers_ref = ONE_SHOT_TIMER.init([one_shot_timer]);
    esp_hal_embassy::init(&clocks, timers_ref);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    // keys
    let sw1_key = Input::new(io.pins.gpio7, Pull::Up);
    let sw2_key = Input::new(io.pins.gpio6, Pull::Up);
    // ssd1306 display
    let scl = io.pins.gpio2;
    let sda = io.pins.gpio3;
    let i2c = I2C::new(peripherals.I2C0, sda, scl, 400.kHz(), &clocks, None);

    spawner.spawn(display_run(i2c)).ok();
    spawner.spawn(sw1_run(sw1_key)).ok();
    spawner.spawn(sw2_run(sw2_key)).ok();
    let mut count = 0;
    loop {
        println!("count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(1000)).await;
    }
}

fn alloc() {
    // -------- Setup Allocator --------
    const HEAP_SIZE: usize = 60 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    #[global_allocator]
    static ALLOCATOR: embedded_alloc::Heap = embedded_alloc::Heap::empty();
    unsafe {
        ALLOCATOR.init(
            &mut HEAP as *const u8 as usize,
            core::mem::size_of_val(&HEAP),
        )
    };
}

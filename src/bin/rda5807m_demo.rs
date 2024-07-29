#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
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
use rda5807m::{Address, Rda5708m};
use shared_bus::{BusManagerSimple, I2cProxy, NullMutex};
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, DisplaySize128x64, I2CInterface};
use ssd1306::{I2CDisplayInterface, Ssd1306};
use static_cell::StaticCell;

use esp32c3_fm::ec11::ec11_detection;
use esp32c3_fm::event::{key_detection, EventType};

static ONE_SHOT_TIMER: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, (u8, EventType), 64> = Channel::new();

#[embassy_executor::task]
async fn ec11_run(
    mut ec11_a: Input<'static, GpioPin<4>>,
    mut ec11_b: Input<'static, GpioPin<5>>,
    mut ec11_key: Input<'static, GpioPin<1>>,
) {
    ec11_detection(
        &mut ec11_a,
        &mut ec11_b,
        &mut ec11_key,
        |event_type, speed| {
            println!("event type: {:?}, speed: {}", event_type, speed);
            CHANNEL.try_send((1, event_type)).ok();
        },
    )
    .await;
}

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

fn draw_text(
    display: &mut Ssd1306<
        I2CInterface<I2cProxy<NullMutex<I2C<'static, I2C0, Blocking>>>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    text: &str,
) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    Text::with_baseline(text, Point::new(0, 19), text_style, Baseline::Top)
        .draw(display)
        .expect("draw text fail");
    display.flush().expect("flush display fail");
    display.clear(BinaryColor::Off).expect("clear display fail");
}

#[embassy_executor::task]
async fn display_run(i2c: I2C<'static, I2C0, Blocking>) {
    let i2c_bus_manager = BusManagerSimple::new(i2c);
    // rda5807m
    let mut rda5807m = Rda5708m::new(i2c_bus_manager.acquire_i2c(), Address::default());
    match rda5807m.start() {
        Ok(_) => {
            println!("start rda5807m success!");
        }
        Err(e) => {
            println!("start rda5807m err, {:?}", e);
        }
    }

    // ssd1306 display
    let interface = I2CDisplayInterface::new(i2c_bus_manager.acquire_i2c());
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().expect("init display fail");
    display.flush().expect("flush display fail");
    display.clear(BinaryColor::Off).expect("clear display fail");
    loop {
        let msg = CHANNEL.receive().await;
        match msg {
            (7, EventType::KeyShort) => {
                // pre
                match rda5807m.seek_up(true) {
                    Ok(_) => {
                        println!("seek up success!");
                    }
                    Err(e) => {
                        println!("seek up err, {:?}", e);
                    }
                }
            }
            (6, EventType::KeyShort) => {
                // next
                match rda5807m.seek_down(true) {
                    Ok(_) => {
                        println!("seek down success!");
                    }
                    Err(e) => {
                        println!("seek down err, {:?}", e);
                    }
                }
            }
            (1, EventType::KeyShort) => {
                let freq = rda5807m.get_frequency().unwrap_or(0);
                draw_text(&mut display, format!("freq: {}", freq).as_str());
            }
            (1, EventType::EC11Front) => match rda5807m.volume_up() {
                Ok(_) => {
                    println!("volume up success!");
                }
                Err(e) => {
                    println!("volume up err, {:?}", e);
                }
            },
            (1, EventType::EC11Back) => match rda5807m.volume_down() {
                Ok(_) => {
                    println!("volume down success!");
                }
                Err(e) => {
                    println!("volume down err, {:?}", e);
                }
            },
            (_io, _event_type) => {}
        }
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
    println!("Start!");
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // keys
    let sw1_key = Input::new(io.pins.gpio7, Pull::Up);
    let sw2_key = Input::new(io.pins.gpio6, Pull::Up);
    // ec11
    let ec11_a = Input::new(io.pins.gpio4, Pull::Up);
    let ec11_b = Input::new(io.pins.gpio5, Pull::Up);
    let ec11_key = Input::new(io.pins.gpio1, Pull::Up);

    // i2c
    let scl = io.pins.gpio2;
    let sda = io.pins.gpio3;
    let i2c = I2C::new(peripherals.I2C0, sda, scl, 400.kHz(), &clocks, None);
    // start
    spawner.spawn(display_run(i2c)).ok();
    spawner.spawn(sw1_run(sw1_key)).ok();
    spawner.spawn(sw2_run(sw2_key)).ok();
    spawner.spawn(ec11_run(ec11_a, ec11_b, ec11_key)).ok();

    loop {
        Timer::after(Duration::from_millis(5_000)).await;
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

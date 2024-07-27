#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp32c3_fm::event::key_detection;
use esp_backtrace as _;
use esp_hal::gpio::{GpioPin, Input, Io, Pull};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::{ErasedTimer, OneShotTimer};
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl};
use esp_println::println;
use static_cell::StaticCell;

static ONE_SHOT_TIMER: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();

#[embassy_executor::task]
async fn sw1_run(mut sw1_key: Input<'static, GpioPin<7>>) {
    loop {
        sw1_key.wait_for_falling_edge().await;
        key_detection::<GpioPin<7>>(&sw1_key, |event_type| {
            println!("event_type:{:?}", event_type);
        })
        .await;
    }
}
#[embassy_executor::task]
async fn sw2_run(mut sw2_key: Input<'static, GpioPin<6>>) {
    loop {
        sw2_key.wait_for_falling_edge().await;
        key_detection::<GpioPin<6>>(&sw2_key, |event_type| {
            println!("event_type:{:?}", event_type);
        })
        .await;
    }
}

#[main]
async fn main(spawner: Spawner) {
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
    let sw1_key = Input::new(io.pins.gpio7, Pull::Up);
    let sw2_key = Input::new(io.pins.gpio6, Pull::Up);

    spawner.spawn(sw1_run(sw1_key)).ok();
    spawner.spawn(sw2_run(sw2_key)).ok();
    println!("Start!");
}

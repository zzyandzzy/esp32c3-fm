#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl,
};
use esp_hal::timer::{ErasedTimer, OneShotTimer};
use esp_hal::timer::timg::TimerGroup;
use static_cell::StaticCell;

static ONE_SHOT_TIMER: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    esp_println::println!("Init!");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // initialize the timer(s)
    let timer_group = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let one_shot_timer = OneShotTimer::new(timer_group.timer0.into());
    let timers_ref = ONE_SHOT_TIMER.init([one_shot_timer]);

    esp_hal_embassy::init(&clocks, timers_ref);
    esp_println::println!("Start!");

    spawner.spawn(run()).ok();

    loop {
        esp_println::println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

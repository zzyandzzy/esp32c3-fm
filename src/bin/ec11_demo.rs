#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp32c3_fm::ec11;
#[allow(unused)]
use esp_backtrace as _;
use esp_hal::gpio::{Input, Io, Pull};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::{ErasedTimer, OneShotTimer};
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl};
use esp_println::println;
use static_cell::StaticCell;

static ONE_SHOT_TIMER: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();

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
    let ec11_a = Input::new(io.pins.gpio4, Pull::Up);
    let ec11_b = Input::new(io.pins.gpio5, Pull::Up);
    let ec11_key = Input::new(io.pins.gpio1, Pull::Up);

    spawner.spawn(ec11::task(ec11_a, ec11_b, ec11_key)).ok();
    println!("Start!");
}

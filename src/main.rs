#![no_std]
#![no_main]

extern crate alloc;

mod ec11;
mod event;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::{ErasedTimer, OneShotTimer};
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl};
use esp_hal::gpio::{AnyInput, Input, Io, Pull};
use esp_println::println;
use static_cell::StaticCell;

static ONE_SHOT_TIMER: StaticCell<[OneShotTimer<ErasedTimer>; 1]> = StaticCell::new();

#[embassy_executor::task]
async fn run() {
    loop {
        println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    alloc();
    esp_println::logger::init_logger_from_env();

    println!("Init!");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // initialize the timer(s)
    let timer_group = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let one_shot_timer = OneShotTimer::new(timer_group.timer0.into());
    let timers_ref = ONE_SHOT_TIMER.init([one_shot_timer]);

    esp_hal_embassy::init(&clocks, timers_ref);
    println!("Start!");

    // spawner.spawn(run()).ok();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let ec11_a = Input::new(io.pins.gpio4, Pull::Up);
    let ec11_b = Input::new(io.pins.gpio5, Pull::Up);
    let ec11_key = Input::new(io.pins.gpio1, Pull::Up);
    spawner.spawn(ec11::task(ec11_a, ec11_b, ec11_key)).ok();

    loop {
        Timer::after(Duration::from_millis(10)).await;
    }
}

fn alloc() {
    // -------- Setup Allocator --------
    const HEAP_SIZE: usize = 60 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    #[global_allocator]
    static ALLOCATOR: embedded_alloc::Heap = embedded_alloc::Heap::empty();
    unsafe { ALLOCATOR.init(&mut HEAP as *const u8 as usize, core::mem::size_of_val(&HEAP)) };
}
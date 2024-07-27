use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::{Input, InputPin};
use esp_println::println;

#[derive(Eq, PartialEq, Debug)]
pub enum EventType {
    KeyShort,
    KeyLongStart,
    KeyLongIng,
    KeyLongEnd,
}

pub async fn toggle_event(event_type: EventType, ms: u64) {
    println!("event_type:{:?} {}", event_type, ms);
}

pub async fn key_detection<P, F>(key: &Input<'static, P>, mut callback: F)
where
    P: InputPin,
    F: FnMut(EventType) -> (),
{
    let begin_ms = Instant::now().as_millis();
    let mut is_long = false;
    loop {
        let mut is_low_times = 0;
        for _i in 0..100 {
            if key.is_low() {
                is_low_times += 1;
            }
        }
        if is_low_times > 80 {
            //按下
            let current = Instant::now().as_millis();
            if current - begin_ms > 500 {
                //长时间按下
                if !is_long {
                    is_long = true;
                    callback(EventType::KeyLongStart);
                } else {
                    callback(EventType::KeyLongIng);
                }
            }
        } else if is_low_times < 2 {
            //释放
            if is_long {
                //长时间按下后释放
                callback(EventType::KeyLongEnd);
                return;
            } else {
                //短时按下，等几ms 看是否有下一次按下，如有则是双击
                loop {
                    callback(EventType::KeyShort);
                    return;
                }
            }
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}

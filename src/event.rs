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

pub async fn key_detection<P>(key: &Input<'static, P>, callback: fn(EventType))
where
    P: InputPin,
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
                    // toggle_event(EventType::KeyLongStart, current).await;
                } else {
                    callback(EventType::KeyLongIng);
                    // toggle_event(EventType::KeyLongIng, current).await;
                }
            }
        } else if is_low_times < 2 {
            //释放
            if is_long {
                //长时间按下后释放
                callback(EventType::KeyLongEnd);
                // toggle_event(EventType::KeyLongEnd, current).await;
                return;
            } else {
                //短时按下，等几ms 看是否有下一次按下，如有则是双击
                loop {
                    callback(EventType::KeyShort);
                    // toggle_event(EventType::KeyShort, current).await;
                    return;
                }
            }
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}

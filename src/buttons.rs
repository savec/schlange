use embassy_nrf::gpio::{AnyPin, Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

struct Debouncer<'a> {
    input: Input<'a>,
    debounce: Duration,
}

impl<'a> Debouncer<'a> {
    pub fn new(input: Input<'a>, debounce: Duration) -> Self {
        Self { input, debounce }
    }

    async fn debounce(&mut self) -> Level {
        loop {
            let l1 = self.input.get_level();

            self.input.wait_for_any_edge().await;

            Timer::after(self.debounce).await;

            let l2 = self.input.get_level();
            if l1 != l2 {
                break l2;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonCode {
    PressedA,
    PressedB,
}

pub static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, ButtonCode> = Signal::new();

pub fn try_get_code() -> Option<ButtonCode> {
    BUTTON_SIGNAL.try_take()
}

#[embassy_executor::task(pool_size = 2)]
pub async fn btn_task(btn: AnyPin, btn_signal: ButtonCode) {
    let mut btn = Debouncer::new(Input::new(btn, Pull::None), Duration::from_millis(20));
    loop {
        if btn.debounce().await == Level::Low {
            BUTTON_SIGNAL.signal(btn_signal);
        }
    }
}

use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::LedPins;

// go for the signal because we can, 50-bytes transaction per 100-500 ms is not of a big deal
pub static FRAME_SIGNAL: Signal<CriticalSectionRawMutex, Frame> = Signal::new();

struct LedMatrix<'a> {
    cols: [Output<'a>; 5],
    rows: [Output<'a>; 5],
    frame: Frame,
}

impl<'a> LedMatrix<'a> {
    fn new(pins: LedPins) -> Self {
        LedMatrix {
            rows: [
                Output::new(pins.row1_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.row2_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.row3_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.row4_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.row5_pin, Level::Low, OutputDrive::Standard),
            ],
            cols: [
                Output::new(pins.col1_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.col2_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.col3_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.col4_pin, Level::Low, OutputDrive::Standard),
                Output::new(pins.col5_pin, Level::Low, OutputDrive::Standard),
            ],
            frame: Default::default(),
        }
    }

    fn set_frame(&mut self, frame: Frame) {
        self.frame = frame;
    }

    async fn drive(&mut self) {
        for (frame_rows, col_led) in self.frame.buffer.iter_mut().zip(self.cols.iter_mut()) {
            col_led.set_low();
            for (frame_row, row_led) in frame_rows.iter_mut().zip(self.rows.iter_mut()) {
                match *frame_row {
                    PixelState::Off => {
                        Timer::after_micros(1000).await;
                    }
                    PixelState::Solid(l) => {
                        row_led.set_high();
                        Timer::after_micros(l as u64).await;
                        row_led.set_low();
                        Timer::after_micros(1000 - l as u64).await;
                    }
                    PixelState::Blinking(mut state) => {
                        row_led.set_high();
                        Timer::after_micros(state.brightness as u64).await;
                        row_led.set_low();
                        Timer::after_micros(1000 - state.brightness as u64).await;
                        state.process();
                        *frame_row = PixelState::Blinking(state);
                    }
                };
            }
            col_led.set_high();
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PixelState {
    #[default]
    Off,
    Solid(i32),
    Blinking(BlinkingPixel<0, 1000, 50>),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct BlinkingPixel<const MIN: i32, const MAX: i32, const STEP: i32> {
    fading: bool,
    brightness: i32,
}

impl<const MIN: i32, const MAX: i32, const STEP: i32> BlinkingPixel<MIN, MAX, STEP> {
    pub fn new() -> Self {
        BlinkingPixel {
            fading: true,
            brightness: MAX,
        }
    }

    fn process(&mut self) {
        self.fading = match self.fading {
            true => {
                self.brightness -= STEP;
                if self.brightness > MIN {
                    true
                } else {
                    self.brightness = MIN;
                    false
                }
            }
            false => {
                self.brightness += STEP;
                if self.brightness < MAX {
                    false
                } else {
                    self.brightness = MAX;
                    true
                }
            }
        };
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Frame {
    pub buffer: [[PixelState; 5]; 5],
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            ..Default::default()
        }
    }
}

pub fn send_frame(frame: Frame) {
    FRAME_SIGNAL.signal(frame);
}

#[embassy_executor::task]
pub async fn led_task(pins: LedPins) {
    let mut led_matrix = LedMatrix::new(pins);
    loop {
        if let Some(frame) = FRAME_SIGNAL.try_take() {
            led_matrix.set_frame(frame);
        }
        led_matrix.drive().await;
    }
}

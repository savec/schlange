use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::LedPins;

// go for the signal because we can, 50-bytes transaction per 100-500 ms is not of a big deal
pub static SNAPSHOT_SIGNAL: Signal<CriticalSectionRawMutex, Snapshot<5, 5>> = Signal::new();

struct LedMatrix<'a, const NCOLS: usize, const NROWS: usize> {
    cols: [Output<'a>; NCOLS],
    rows: [Output<'a>; NROWS],
    frame: Frame<5, 5>,
}

impl<'a> LedMatrix<'a, 5, 5> {
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
            frame: Frame::new(),
        }
    }

    fn set_frame(&mut self, frame: Frame<5, 5>) {
        self.frame = frame;
    }

    fn get_frame(&self) -> &Frame<5, 5> {
        &self.frame
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
                        if state.brightness > 0 {
                            row_led.set_high();
                            Timer::after_micros(state.brightness as u64).await;
                            row_led.set_low();
                        }
                        Timer::after_micros(1000 - state.brightness as u64).await;
                        state.process();
                        *frame_row = PixelState::Blinking(state);
                    }
                    PixelState::Fading(mut state) => {
                        if state.brightness > 0 {
                            row_led.set_high();
                            Timer::after_micros(state.brightness as u64).await;
                            row_led.set_low();
                        }
                        Timer::after_micros(1000 - state.brightness as u64).await;
                        state.process();
                        *frame_row = PixelState::Fading(state);
                    }
                    PixelState::FlareUp(mut state) => {
                        if state.brightness > 0 {
                            row_led.set_high();
                            Timer::after_micros(state.brightness as u64).await;
                            row_led.set_low();
                        }
                        Timer::after_micros(1000 - state.brightness as u64).await;
                        state.process();
                        *frame_row = PixelState::FlareUp(state);
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
    Solid(u16),
    Blinking(BlinkingPixel<0, 1000, 100>),
    Fading(FadingPixel<1000, 50>),
    FlareUp(FlareUpPixel<1000, 50>),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct FlareUpPixel<const MAX: i16, const STEP: i16> {
    brightness: i16,
}

impl<const MAX: i16, const STEP: i16> FlareUpPixel<MAX, STEP> {
    pub fn new() -> Self {
        FlareUpPixel { brightness: 0 }
    }

    fn process(&mut self) {
        if self.brightness < MAX {
            self.brightness += STEP;
        } else {
            self.brightness = MAX;
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct FadingPixel<const MAX: i16, const STEP: i16> {
    brightness: i16,
}

impl<const MAX: i16, const STEP: i16> FadingPixel<MAX, STEP> {
    pub fn new() -> Self {
        FadingPixel { brightness: MAX }
    }

    fn process(&mut self) {
        if self.brightness > STEP {
            self.brightness -= STEP;
        } else {
            self.brightness = 0;
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct BlinkingPixel<const MIN: i16, const MAX: i16, const STEP: i16> {
    fading: bool,
    brightness: i16,
}

impl<const MIN: i16, const MAX: i16, const STEP: i16> BlinkingPixel<MIN, MAX, STEP> {
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

#[derive(Debug, Clone, Copy)]
struct Frame<const NCOLS: usize, const NROWS: usize> {
    pub buffer: [[PixelState; NCOLS]; NROWS],
}

impl Frame<5, 5> {
    pub fn new() -> Self {
        Frame {
            buffer: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Snapshot<const NCOLS: usize, const NROWS: usize> {
    pub buffer: [[CellState; NCOLS]; NROWS],
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    #[default]
    Empty,
    SnakeHead,
    SnakeTail,
    Food,
    AnimationStatic(u16),
    AnimationFadingInterFrame,
    AnimationFlareUpInterFrame,
    AnimationBlinking,
}

impl Snapshot<5, 5> {
    pub fn new() -> Self {
        Snapshot {
            buffer: Default::default(),
        }
    }
}

pub fn send_snapshot(snapshot: &Snapshot<5, 5>) {
    SNAPSHOT_SIGNAL.signal(*snapshot);
}

#[derive(Debug)]
struct Render {
    prev_snapshot: Snapshot<5, 5>,
}

impl Render {
    fn new() -> Self {
        Render {
            prev_snapshot: Snapshot::new(),
        }
    }
    fn render(&mut self, snapshot: Snapshot<5, 5>, current_frame: &Frame<5, 5>) -> Frame<5, 5> {
        let mut frame = Frame::new();
        for (col, frame_cols) in frame.buffer.iter_mut().enumerate() {
            for (row, frame_pixel) in frame_cols.iter_mut().enumerate() {
                *frame_pixel = match snapshot.buffer[col][row] {
                    CellState::Empty => PixelState::Off,
                    CellState::SnakeHead => PixelState::Solid(1000),
                    CellState::SnakeTail => PixelState::Solid(100),
                    CellState::Food => {
                        if self.prev_snapshot.buffer[col][row] == CellState::Food {
                            current_frame.buffer[col][row]
                        } else {
                            PixelState::Blinking(BlinkingPixel::new())
                        }
                    }
                    CellState::AnimationStatic(brightness) => PixelState::Solid(brightness),
                    CellState::AnimationFadingInterFrame => {
                        if self.prev_snapshot.buffer[col][row]
                            == CellState::AnimationFadingInterFrame
                        {
                            current_frame.buffer[col][row]
                        } else {
                            PixelState::Fading(FadingPixel::new())
                        }
                    }
                    CellState::AnimationBlinking => PixelState::Blinking(BlinkingPixel::new()),
                    CellState::AnimationFlareUpInterFrame => {
                        if self.prev_snapshot.buffer[col][row]
                            == CellState::AnimationFlareUpInterFrame
                        {
                            current_frame.buffer[col][row]
                        } else {
                            PixelState::FlareUp(FlareUpPixel::new())
                        }
                    }
                }
            }
        }
        self.prev_snapshot = snapshot;
        frame
    }
}

#[embassy_executor::task]
pub async fn led_task(pins: LedPins) {
    let mut led_matrix = LedMatrix::new(pins);
    let mut render = Render::new();
    loop {
        if let Some(frame) = SNAPSHOT_SIGNAL.try_take() {
            led_matrix.set_frame(render.render(frame, led_matrix.get_frame()));
        }
        led_matrix.drive().await;
    }
}

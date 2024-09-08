#![no_std]
#![no_main]

mod fmt;
use core::panic;

use defmt::info;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use fmt::unwrap;
use micro_rand::Random;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_nrf::{
    config::Config,
    gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull},
};
use embassy_nrf::{config::HfclkSource, peripherals};

assign_resources! {
    led_pins: LedPins{
        col1_pin: P0_28,
        col2_pin: P0_11,
        col3_pin: P0_31,
        col4_pin: P1_05,
        col5_pin: P0_30,
        row1_pin: P0_21,
        row2_pin: P0_22,
        row3_pin: P0_15,
        row4_pin: P0_24,
        row5_pin: P0_19,
    }
    btn_a_pin: ButtonAPin {
        btn_pin: P0_14,
    }
    btn_b_pin: ButtonBPin {
        btn_pin: P0_23,
    }
    // add more resources to more structs if needed, for example defining one struct for each task
}
// go for the signal because we can, 50-bytes transaction per 100-500 ms is not of a big deal
static FRAME_SIGNAL: Signal<CriticalSectionRawMutex, Frame> = Signal::new();

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
enum PixelState {
    #[default]
    Off,
    Solid(i32),
    Blinking(BlinkingPixel<0, 1000, 50>),
}

#[derive(Default, Debug, Clone, Copy)]
struct BlinkingPixel<const MIN: i32, const MAX: i32, const STEP: i32> {
    fading: bool,
    brightness: i32,
}

impl<const MIN: i32, const MAX: i32, const STEP: i32> BlinkingPixel<MIN, MAX, STEP> {
    fn new() -> Self {
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
struct Frame {
    buffer: [[PixelState; 5]; 5],
}

impl Frame {
    fn new() -> Self {
        Frame {
            ..Default::default()
        }
    }
}

#[embassy_executor::task]
async fn led_task(pins: LedPins) {
    let mut led_matrix = LedMatrix::new(pins);
    loop {
        if let Some(frame) = FRAME_SIGNAL.try_take() {
            led_matrix.set_frame(frame);
        }
        led_matrix.drive().await;
    }
}

struct Debouncer<'a> {
    input: Input<'a>,
    debounce: Duration,
}

impl<'a> Debouncer<'a> {
    pub fn new(input: Input<'a>, debounce: Duration) -> Self {
        Self { input, debounce }
    }

    pub async fn debounce(&mut self) -> Level {
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
enum ButtonCode {
    PressedA,
    PressedB,
}

static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, ButtonCode> = Signal::new();

#[embassy_executor::task(pool_size = 2)]
async fn btn_task(btn: AnyPin, btn_signal: ButtonCode) {
    let mut btn = Debouncer::new(Input::new(btn, Pull::None), Duration::from_millis(20));
    loop {
        if btn.debounce().await == Level::Low {
            BUTTON_SIGNAL.signal(btn_signal);
        }
    }
}

enum Direction {
    North,
    Ost,
    South,
    West,
}

#[derive(Default, Clone, Copy)]
struct Coordinate {
    row: u8,
    col: u8,
}

struct RingBuffer<T, const CAP: usize> {
    rb: [T; CAP],
    head: usize,
    tail: usize,
}

#[derive(Debug)]
enum RbError {
    NoMoreSpace,
    IsEmpty,
}

impl<T, const CAP: usize> RingBuffer<T, CAP>
where
    T: Default + Copy,
{
    fn new() -> Self {
        RingBuffer {
            rb: [Default::default(); CAP],
            head: 0,
            tail: 0,
        }
    }

    fn put(&mut self, elem: T) -> Result<(), RbError> {
        if (self.head + 1) % CAP == self.tail {
            Err(RbError::NoMoreSpace)
        } else {
            self.rb[self.head] = elem;
            self.head = (self.head + 1) % CAP;
            Ok(())
        }
    }

    fn get(&mut self) -> Result<T, RbError> {
        if self.head == self.tail {
            Err(RbError::IsEmpty)
        } else {
            let elem = self.rb[self.tail];
            self.tail = (self.tail + 1) % CAP;
            Ok(elem)
        }
    }

    fn len(&self) -> usize {
        (self.head + CAP - self.tail) % CAP
    }

    fn capacity(&self) -> usize {
        CAP
    }

    fn peek_head(&self) -> T {
        self.rb[(self.head + CAP - 1) % CAP]
    }

    fn iter(&self) -> RingBufferIterator<T, CAP> {
        RingBufferIterator {
            rb: self,
            tail: self.tail,
        }
    }
}

impl<'a, T, const CAP: usize> Iterator for RingBufferIterator<'a, T, CAP> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rb.head == self.tail {
            None
        } else {
            let elem = &self.rb.rb[self.tail];
            self.tail = (self.tail + 1) % CAP;
            Some(elem)
        }
    }
}

struct RingBufferIterator<'a, T, const CAP: usize> {
    rb: &'a RingBuffer<T, CAP>,
    tail: usize,
}

struct Game {
    snake: RingBuffer<Coordinate, 25>,
    direction: Direction,
    food: Coordinate,
    rnd: Random,
}

#[derive(Debug)]
enum SnakeError {
    Fatal,
}

enum MoveResult {
    Trivial,
    BiteYourself,
    AteSomething,
    ItIsDone,
}

impl Game {
    fn new() -> Self {
        let mut snake: RingBuffer<Coordinate, 25> = RingBuffer::new();
        snake.put(Coordinate { row: 2, col: 2 }).unwrap();
        // TODO add seed
        let rnd = Random::new(1234);

        let mut game = Game {
            direction: Direction::North,
            snake,
            rnd,
            food: Default::default(),
        };

        game.give_food();
        game
    }

    fn is_snake(&self, coordinate: Coordinate) -> bool {
        for c in self.snake.iter() {
            if c.row == coordinate.row && c.col == coordinate.col {
                return true;
            }
        }
        false
    }

    fn give_food(&mut self) {
        let base = self.snake.capacity() - self.snake.len();
        let random = self.rnd.next_int_i32(0, base as i32 - 1);
        info!("random: {}", random);
        let mut count = 0;
        for row in 0..5 {
            for col in 0..5 {
                let coordinate = Coordinate { row, col };
                if self.is_snake(coordinate) {
                    continue;
                }
                if count == random {
                    self.food = coordinate;
                    return;
                }
                count += 1;
            }
        }
        panic!();
    }

    fn update_direction(&mut self, input: ButtonCode) {
        match input {
            ButtonCode::PressedA => {
                self.direction = match self.direction {
                    Direction::North => Direction::West,
                    Direction::Ost => Direction::North,
                    Direction::South => Direction::Ost,
                    Direction::West => Direction::South,
                }
            }
            ButtonCode::PressedB => {
                self.direction = match self.direction {
                    Direction::North => Direction::Ost,
                    Direction::Ost => Direction::South,
                    Direction::South => Direction::West,
                    Direction::West => Direction::North,
                }
            }
        }
    }

    fn get_new_head_coordinate(&self) -> Coordinate {
        let head = self.snake.peek_head();
        // info!("head is {} {}", head.row, head.col);
        match self.direction {
            Direction::Ost => Coordinate {
                row: head.row,
                col: (head.col + 1) % 5,
            },
            Direction::South => Coordinate {
                col: head.col,
                row: (head.row + 1) % 5,
            },
            Direction::West => Coordinate {
                row: head.row,
                col: (head.col + 5 - 1) % 5,
            },
            Direction::North => Coordinate {
                col: head.col,
                row: (head.row + 5 - 1) % 5,
            },
        }
    }

    fn do_move(&mut self) -> Result<MoveResult, SnakeError> {
        let new_head = self.get_new_head_coordinate();
        // info!("new_head is {} {}", new_head.row, new_head.col);
        if self.food.row == new_head.row && self.food.col == new_head.col {
            let result = match self.snake.put(new_head) {
                Ok(_) => Ok(MoveResult::AteSomething),
                Err(_) => Ok(MoveResult::ItIsDone),
            };
            self.give_food();
            return result;
        }
        if self.is_snake(new_head) {
            return Ok(MoveResult::BiteYourself);
        } else {
            // check if there is food in position of new head. If there is - then don't cut the tail
            let _ = self.snake.get().map_err(|_| SnakeError::Fatal)?;
        }

        match self.snake.put(new_head) {
            Ok(_) => Ok(MoveResult::Trivial),
            Err(_) => Ok(MoveResult::ItIsDone),
        }
    }

    fn get_frame(&mut self) -> Frame {
        let mut frame = Frame::new();
        for c in self.snake.iter() {
            frame.buffer[c.col as usize][c.row as usize] = PixelState::Solid(100);
        }
        let head = self.snake.peek_head();
        frame.buffer[head.col as usize][head.row as usize] = PixelState::Solid(1000);
        frame.buffer[self.food.col as usize][self.food.row as usize] =
            PixelState::Blinking(BlinkingPixel::new());

        frame
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config: Config = Default::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    let p = embassy_nrf::init(config);
    let r = split_resources!(p);
    unwrap!(spawner.spawn(led_task(r.led_pins)));
    unwrap!(spawner.spawn(btn_task(r.btn_a_pin.btn_pin.into(), ButtonCode::PressedA)));
    unwrap!(spawner.spawn(btn_task(r.btn_b_pin.btn_pin.into(), ButtonCode::PressedB)));
    loop {
        let mut game = Game::new();

        info!("Schhhh");
        loop {
            Timer::after_millis(500).await;
            if let Some(btn_signal) = BUTTON_SIGNAL.try_take() {
                game.update_direction(btn_signal);
            }
            if let Ok(res) = game.do_move() {
                match res {
                    MoveResult::BiteYourself => {
                        info!("BiteYourself");
                        break;
                    }
                    MoveResult::AteSomething => (),
                    MoveResult::ItIsDone => {
                        info!("Done");
                        break;
                    }
                    _ => (),
                }
            } else {
                info!("Fatal");
                break;
            }
            FRAME_SIGNAL.signal(game.get_frame());
        }
    }
}

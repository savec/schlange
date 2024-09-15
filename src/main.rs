#![no_std]
#![no_main]

mod animation;
mod buttons;
mod difficulty;
mod fmt;
mod led;
mod rb;

use crate::buttons::btn_task;
use crate::buttons::ButtonCode;
use crate::led::led_task;

use animation::DEFEAT;
use animation::INTRO;
use animation::VICTORY;
use buttons::try_get_code;
use difficulty::DifficultySelector;
use embassy_time::Timer;
use fmt::unwrap;
use heapless::FnvIndexSet;
use led::send_snapshot;
use led::CellState;
use led::Snapshot;
use micro_rand::Random;
use rb::RingBuffer;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_nrf::config::Config;
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
enum Direction {
    North,
    Ost,
    South,
    West,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    row: u8,
    col: u8,
}

struct Game {
    snake: RingBuffer<Coordinate, 25>,
    no_snake: FnvIndexSet<Coordinate, 32>,
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
    Win,
}

impl Game {
    fn new() -> Self {
        let snake: RingBuffer<Coordinate, 25> = RingBuffer::new();
        let mut no_snake: FnvIndexSet<Coordinate, 32> = FnvIndexSet::new();
        for row in 0..5 {
            for col in 0..5 {
                let _ = no_snake.insert(Coordinate { row, col });
            }
        }
        let rnd = Random::new(embassy_time::Instant::now().as_ticks() as i64);
        let mut game = Game {
            direction: Direction::North,
            snake,
            no_snake,
            rnd,
            food: Default::default(),
        };

        game.snake_add_head(Coordinate { row: 2, col: 2 }).unwrap();
        game.give_food();
        game
    }

    fn is_snake(&self, coordinate: Coordinate) -> bool {
        !self.no_snake.contains(&coordinate)
    }

    fn give_food(&mut self) {
        let empty_count = self.no_snake.len() as i32;
        if empty_count > 0 {
            let random = self.rnd.next_int_i32(0, empty_count - 1);
            self.food = *self.no_snake.iter().nth(random as usize).unwrap();
        }
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

    fn is_food(&self, coordinate: Coordinate) -> bool {
        self.food == coordinate
    }

    fn do_move(&mut self) -> Result<MoveResult, SnakeError> {
        let new_head = self.get_new_head_coordinate();
        if self.is_snake(new_head) {
            Ok(MoveResult::BiteYourself)
        } else {
            let result = self.snake_add_head(new_head);
            if self.is_food(new_head) {
                self.give_food();
            } else {
                self.snake_cut_tail()?;
            }
            result
        }
    }

    fn get_snapshot(&mut self) -> Snapshot<5, 5> {
        let mut snapshot = Snapshot::new();

        let mut snake_iter = self.snake.iter();
        let head = snake_iter.next().unwrap();
        snapshot.buffer[head.col as usize][head.row as usize] = CellState::SnakeHead;
        for tail in snake_iter {
            snapshot.buffer[tail.col as usize][tail.row as usize] = CellState::SnakeTail;
        }
        snapshot.buffer[self.food.col as usize][self.food.row as usize] = CellState::Food;

        snapshot
    }

    fn snake_add_head(&mut self, coordinate: Coordinate) -> Result<MoveResult, SnakeError> {
        self.no_snake.remove(&coordinate);
        match self.snake.put(coordinate) {
            Ok(_) => Ok(MoveResult::Trivial),
            Err(_) => Ok(MoveResult::Win),
        }
    }

    fn snake_cut_tail(&mut self) -> Result<MoveResult, SnakeError> {
        let tail = self.snake.get().map_err(|_| SnakeError::Fatal)?;
        let _ = self.no_snake.insert(tail);
        Ok(MoveResult::Trivial)
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
        let mut difficulty_selector = DifficultySelector::new();
        send_snapshot(&difficulty_selector.get_snapshot());
        loop {
            if let Some(btn_signal) = try_get_code() {
                if let Some(_) = difficulty_selector.is_choice_made(btn_signal) {
                    break;
                }
                send_snapshot(&difficulty_selector.get_snapshot());
            }
            Timer::after_millis(100).await;
        }
        INTRO.playback().await;
        let mut game = Game::new();
        loop {
            if let Some(btn_signal) = try_get_code() {
                game.update_direction(btn_signal);
            }
            let res = game.do_move().unwrap();
            match res {
                MoveResult::BiteYourself => {
                    DEFEAT.playback().await;
                    break;
                }
                MoveResult::Win => {
                    VICTORY.playback().await;
                    break;
                }
                _ => (),
            }
            send_snapshot(&game.get_snapshot());
            Timer::after_millis(difficulty_selector.get_turn_delay_ms()).await;
        }
    }
}

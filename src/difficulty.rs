use crate::{
    buttons::ButtonCode,
    led::{CellState, Snapshot},
};

#[derive(Default, Clone, Copy)]
enum Difficulty {
    #[default]
    Easy,
    Normal,
    Hard,
}

pub struct DifficultySelector {
    difficulty: Difficulty,
}

impl DifficultySelector {
    pub fn new() -> Self {
        DifficultySelector {
            difficulty: Default::default(),
        }
    }
    pub fn is_choise_made(&mut self, input: ButtonCode) -> Option<()> {
        match input {
            ButtonCode::PressedA => {
                self.rotate_difficulty();
                None
            }
            ButtonCode::PressedB => Some(()),
        }
    }

    fn rotate_difficulty(&mut self) {
        self.difficulty = match self.difficulty {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        };
    }

    pub fn get_turn_delay_ms(&self) -> u64 {
        match self.difficulty {
            Difficulty::Easy => 750,
            Difficulty::Normal => 500,
            Difficulty::Hard => 250,
        }
    }

    pub fn get_snapshot(&self) -> Snapshot<5, 5> {
        match self.difficulty {
            Difficulty::Easy => Snapshot {
                buffer: [
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                    ],
                ],
            },
            Difficulty::Normal => Snapshot {
                buffer: [
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                ],
            },
            Difficulty::Hard => Snapshot {
                buffer: [
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationStatic(1000),
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                ],
            },
        }
    }
}

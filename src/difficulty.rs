use crate::{
    buttons::ButtonCode,
    led::{CellState, Snapshot},
};

#[derive(Default, Clone, Copy)]
enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Insane,
    Hell,
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
    pub fn is_choice_made(&mut self, input: ButtonCode) -> Option<()> {
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
            Difficulty::Hard => Difficulty::Insane,
            Difficulty::Insane => Difficulty::Hell,
            Difficulty::Hell => Difficulty::Easy,
        };
    }

    pub fn get_turn_delay_ms(&self) -> u64 {
        match self.difficulty {
            Difficulty::Easy => 1000,
            Difficulty::Normal => 500,
            Difficulty::Hard => 400,
            Difficulty::Insane => 300,
            Difficulty::Hell => 200,
        }
    }

    pub fn get_snapshot(&self) -> Snapshot<5, 5> {
        match self.difficulty {
            Difficulty::Easy => Snapshot {
                buffer: [
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                ],
            },
            Difficulty::Normal => Snapshot {
                buffer: [
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                ],
            },
            Difficulty::Hard => Snapshot {
                buffer: [
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                ],
            },
            Difficulty::Insane => Snapshot {
                buffer: [
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                ],
            },
            Difficulty::Hell => Snapshot {
                buffer: [
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                        CellState::AnimationBlinking,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                ],
            },
        }
    }
}

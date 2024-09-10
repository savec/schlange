use crate::led::{self, send_snapshot, CellState, Snapshot};

struct AnimationFrame {
    snapshot: Snapshot<5, 5>,
    delay: u64,
}

pub struct Animation<Sender, const L: usize>
where
    Sender: Fn(&Snapshot<5, 5>),
{
    seq: [AnimationFrame; L],
    cb: Sender,
}

impl<Sender, const L: usize> Animation<Sender, L>
where
    Sender: Fn(&Snapshot<5, 5>),
{
    pub async fn playback(&self) {
        for frame in self.seq.iter() {
            (self.cb)(&frame.snapshot);
            embassy_time::Timer::after_millis(frame.delay).await;
        }
    }
}

type IntroType = Animation<for<'a> fn(&'a led::Snapshot<5, 5>), 3>;

pub static INTRO: IntroType = Animation {
    seq: [
        AnimationFrame {
            snapshot: Snapshot {
                buffer: [
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationFading,
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationFading,
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationFading,
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                        CellState::AnimationFading,
                        CellState::AnimationStatic(1000),
                        CellState::AnimationStatic(1000),
                    ],
                ],
            },
            delay: 300,
        },
        AnimationFrame {
            snapshot: Snapshot {
                buffer: [
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationFading,
                        CellState::Empty,
                        CellState::AnimationFading,
                        CellState::AnimationStatic(1000),
                    ],
                    [
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                        CellState::Empty,
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                        CellState::Empty,
                        CellState::AnimationFading,
                        CellState::AnimationFading,
                    ],
                    [
                        CellState::AnimationStatic(1000),
                        CellState::AnimationFading,
                        CellState::Empty,
                        CellState::AnimationFading,
                        CellState::AnimationStatic(1000),
                    ],
                ],
            },
            delay: 300,
        },
        AnimationFrame {
            snapshot: Snapshot {
                buffer: [
                    [
                        CellState::AnimationFading,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationFading,
                    ],
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
                        CellState::Empty,
                    ],
                    [
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                    ],
                    [
                        CellState::AnimationFading,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::Empty,
                        CellState::AnimationFading,
                    ],
                ],
            },
            delay: 300,
        },
    ],
    cb: send_snapshot,
};

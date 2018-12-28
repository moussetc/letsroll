use rand::rngs::ThreadRng;
use rand::Rng;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum DiceKind {
    Mock(u16),
    NumberedDice(u16),
    Fudge,
}

impl Eq for DiceKind {}

impl fmt::Display for DiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DiceKind::Mock(mock_value) => format!("Mock{}", mock_value),
                DiceKind::NumberedDice(sides) => format!("D{}", sides),
                DiceKind::Fudge => String::from("F"),
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RollResult {
    pub dice: DiceKind,
    pub result: u16,
}

impl RollResult {
    pub fn new(dice: DiceKind, result: u16) -> RollResult {
        RollResult { dice, result }
    }
}
impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.dice {
                DiceKind::Fudge => String::from(match self.result {
                    1 => "\" \"",
                    2 => "\"+\"",
                    _ => "\"-\"",
                }),
                _ => self.result.to_string(),
            }
        )
    }
}
pub trait Roll {
    fn roll(&mut self) -> RollResult;
}

#[doc(hidden)]
pub(crate) struct Mock {
    mock_value: u16,
}

impl Mock {
    pub fn new(mock_value: u16) -> Mock {
        Mock { mock_value }
    }
}

impl Roll for Mock {
    fn roll(&mut self) -> RollResult {
        RollResult {
            dice: DiceKind::Mock(self.mock_value),
            result: self.mock_value,
        }
    }
}

pub struct NumberedDice {
    dice: u16,
    rng: ThreadRng,
}

impl NumberedDice {
    pub fn new(dice: u16) -> NumberedDice {
        NumberedDice {
            dice,
            rng: rand::thread_rng(),
        }
    }
}

impl Roll for NumberedDice {
    fn roll(&mut self) -> RollResult {
        RollResult {
            dice: DiceKind::NumberedDice(self.dice),
            result: self.rng.gen_range(1, self.dice + 1),
        }
    }
}

pub struct FudgeDice {
    rng: ThreadRng,
}

impl FudgeDice {
    pub fn new() -> FudgeDice {
        FudgeDice {
            rng: rand::thread_rng(),
        }
    }
}

impl Roll for FudgeDice {
    fn roll(&mut self) -> RollResult {
        RollResult {
            dice: DiceKind::Fudge,
            result: self.rng.gen_range(1, 4),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::{self, DiceKind, Roll};

    #[test]
    fn dice_kind_comparison() {
        assert_eq!(DiceKind::Mock(10), DiceKind::Mock(10));
        assert_ne!(DiceKind::Mock(10), DiceKind::Mock(20));
        assert_eq!(DiceKind::NumberedDice(10), DiceKind::NumberedDice(10));
        assert_ne!(DiceKind::NumberedDice(10), DiceKind::NumberedDice(30));
        assert_ne!(DiceKind::NumberedDice(10), DiceKind::Mock(10));
    }

    #[test]
    fn mock_generation() {
        let mock_value = 42;
        let mut gen = dice::Mock::new(mock_value);
        let roll = gen.roll();
        match roll.dice {
            DiceKind::Mock(mock) => assert_eq!(mock, mock_value),
            _ => assert!(false, "Wrong dice kind in result roll"),
        }
        assert_eq!(roll.result, mock_value);
    }

    #[test]
    fn numbered_dice_generation() {
        let dice_sides = 42;
        let mut gen = dice::NumberedDice::new(dice_sides);
        let roll = gen.roll();
        match roll.dice {
            DiceKind::NumberedDice(sides) => assert_eq!(sides, dice_sides),
            _ => assert!(false, "Wrong dice kind in result roll"),
        }
        assert!(
            roll.result > 0,
            "Numbered dice generator rolls should be > 0"
        );
        assert!(
            roll.result <= dice_sides,
            "Numbered dice generator rolls should be <= to the number of sides on the dice"
        );
    }
}

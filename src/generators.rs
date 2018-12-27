use rand::rngs::ThreadRng;
use rand::Rng;
use std::fmt;

#[derive(Debug, Clone)]
pub struct RollResult {
    pub dice: u16,
    pub result: u16,
}

impl RollResult {
    pub fn new(dice: u16, result: u16) -> RollResult {
        RollResult { dice, result }
    }
}

#[derive(Debug)]
pub enum DiceKind {
    Mock(u16),
    NumberedDice(u16),
}

impl fmt::Display for DiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                DiceKind::Mock(mock_value) => format!("Mock{}", mock_value),
                DiceKind::NumberedDice(sides) => format!("D{}", sides),
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
            dice: self.mock_value,
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
            dice: self.dice,
            result: self.rng.gen_range(1, self.dice + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generators;
    use crate::generators::Roll;

    #[test]
    fn mock_generation() {
        let mock_value = 42;
        let mut gen = generators::Mock::new(mock_value);
        let roll = gen.roll();
        assert_eq!(
            roll.dice, mock_value,
            "Mock generator didn't use the correct number of dice sides"
        );
        assert_eq!(
            roll.result, mock_value,
            "Mock generator should always roll as specified (expected {})",
            mock_value
        );
    }

    #[test]
    fn numbered_dice_generation() {
        let dice_sides = 42;
        let mut gen = generators::NumberedDice::new(dice_sides);
        let roll = gen.roll();
        assert_eq!(
            roll.dice, dice_sides,
            "Numbered dice generator didn't use the correct number of dice sides"
        );
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

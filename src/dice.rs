use rand::rngs::ThreadRng;
use rand::Rng;
use std::fmt;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;

// TODO continuer de remplacer par les énumérations qui permettent d'éviter les génériques (DiceKind, NumericRoll, TextRoll, etc.)

pub trait Dice {
    type RollResult: Sized + Clone + Copy;
    fn roll(&mut self) -> Self::RollResult;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DiceKind {
    NumericDice(NumericDice),
    TextDice(TextDice),
}

impl fmt::Display for DiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DiceKind::NumericDice(num_dice) => match num_dice {
                    NumericDice::Mock(dice) => format!("Mock{}", dice.mock_value),
                    NumericDice::NumberedDice(dice) => format!("D{}", dice.sides),
                },
                DiceKind::TextDice(text_dice) => match text_dice {
                    TextDice::FudgeDice(dice) => String::from("F"),
                },
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NumericDice {
    Mock(Mock),
    NumberedDice(NumberedDice),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TextDice {
    FudgeDice(FudgeDice),
}

impl Dice for NumericDice {
    type RollResult = Roll<NumericRoll>;

    fn roll(&mut self) -> Self::RollResult {
        match self {
            NumericDice::Mock(mock) => mock.roll(),
            NumericDice::NumberedDice(nd) => nd.roll(),
        }
    }
}

impl Dice for TextDice {
    type RollResult = Roll<TextRoll>;

    fn roll(&mut self) -> Self::RollResult {
        match self {
            TextDice::FudgeDice(fudge) => fudge.roll(),
        }
    }
}

pub enum RollEnum {
    NumericRoll(Roll<NumericRoll>),
    TextRoll(Roll<TextRoll>),
}
impl fmt::Display for RollEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RollEnum::NumericRoll(dice) => write!(f, "{}", dice),
            RollEnum::TextRoll(dice) => write!(f, "{}", dice),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Roll<T: Sized + Clone + Copy> {
    pub result: T,
}
impl<T: Sized + Display + Clone + Copy> fmt::Display for Roll<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.result.to_string(),)
    }
}

/// Type roll result from numbered dice
pub type NumericRoll = u16;
pub type TextRoll = char;

#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Mock {
    pub(crate) mock_value: u16,
}

impl Mock {
    pub fn new(mock_value: u16) -> Mock {
        Mock { mock_value }
    }
}

impl Dice for Mock {
    type RollResult = Roll<NumericRoll>;

    fn roll(&mut self) -> Self::RollResult {
        Roll {
            result: self.mock_value,
        }
    }
}

#[derive(Debug)]
pub struct NumberedDice {
    sides: NumericRoll,
    rng: ThreadRng,
}

impl NumberedDice {
    pub fn new(sides: NumericRoll) -> NumberedDice {
        NumberedDice {
            sides,
            rng: rand::thread_rng(),
        }
    }
}

impl PartialEq for NumberedDice {
    fn eq(&self, other: &NumberedDice) -> bool {
        self.sides == other.sides
    }
}

impl Eq for NumberedDice {}

impl Dice for NumberedDice {
    type RollResult = Roll<NumericRoll>;

    fn roll(&mut self) -> Self::RollResult {
        Roll {
            result: self.rng.gen_range(1, self.sides + 1),
        }
    }
}

impl Hash for NumberedDice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sides.hash(state);
    }
}

#[derive(Debug)]
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

impl Dice for FudgeDice {
    type RollResult = Roll<char>;
    fn roll(&mut self) -> Self::RollResult {
        Roll {
            result: match self.rng.gen_range(1, 4) {
                1 => ' ',
                2 => '+',
                _ => '-',
            },
        }
    }
}

impl PartialEq for FudgeDice {
    fn eq(&self, other: &FudgeDice) -> bool {
        true
    }
}

impl Eq for FudgeDice {}

impl Hash for FudgeDice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO : ugly but all FudgeDice are one and the same...
        1.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::{self, Dice, DiceKind};

    // #[test]
    // fn dice_kind_comparison() {
    //     assert_eq!(DiceKind::Mock(10), DiceKind::Mock(10));
    //     assert_ne!(DiceKind::Mock(10), DiceKind::Mock(20));
    //     assert_eq!(DiceKind::NumberedDice(10), DiceKind::NumberedDice(10));
    //     assert_ne!(DiceKind::NumberedDice(10), DiceKind::NumberedDice(30));
    //     assert_ne!(DiceKind::NumberedDice(10), DiceKind::Mock(10));
    // }

    // #[test]
    // fn mock_generation() {
    //     let mock_value = 42;
    //     let mut gen = dice::Mock::new(mock_value);
    //     let roll = gen.roll();
    //     match roll.dice {
    //         DiceKind::Mock(mock) => assert_eq!(mock, mock_value),
    //         _ => assert!(false, "Wrong dice kind in result roll"),
    //     }
    //     assert_eq!(roll.result, mock_value);
    // }

    // #[test]
    // fn numbered_dice_generation() {
    //     let dice_sides = 42;
    //     let mut gen = dice::NumberedDice::new(dice_sides);
    //     let roll = gen.roll();
    //     match roll.dice {
    //         DiceKind::NumberedDice(sides) => assert_eq!(sides, dice_sides),
    //         _ => assert!(false, "Wrong dice kind in result roll"),
    //     }
    //     assert!(
    //         roll.result > 0,
    //         "Numbered dice generator rolls should be > 0"
    //     );
    //     assert!(
    //         roll.result <= dice_sides,
    //         "Numbered dice generator rolls should be <= to the number of sides on the dice"
    //     );
    // }
}

use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter;

use crate::errors::{Error, ErrorKind};

pub type DiceNumber = u8;
/// Type of roll result for numbered dice (like D20)
pub type NumericRoll = u16;
// Type of roll result for text dice (like fudge)
pub type TextRoll = char;

/// Trait to represent dice that can be rolled to produce values using the [sum](trait.Roll.html#tymethod.roll) method.
pub trait Roll {
    type RollResult;
    fn roll(&self, n: DiceNumber) -> Self::RollResult;
}

/// Get the max value of a numeric dice (for example, the number of sides of a numbered dice)
pub trait GetMaxValue: Roll {
    fn get_max_value(&self) -> NumericRoll;
}

/// Contains a typed dice
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DiceKind {
    NumericKind(NumericDice),
    TextKind(TextDice),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NumericDice {
    ConstDice(ConstDice<NumericRoll>),
    NumberedDice(NumberedDice),
    RepeatingDice(RepeatingDice<NumericRoll>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TextDice {
    FudgeDice(FudgeDice),
    ConstDice(ConstDice<TextRoll>),
    RepeatingDice(RepeatingDice<TextRoll>),
}

impl Roll for NumericDice {
    type RollResult = Vec<NumericRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        match self {
            NumericDice::ConstDice(dice) => dice.roll(n),
            NumericDice::NumberedDice(dice) => dice.roll(n),
            NumericDice::RepeatingDice(dice) => dice.roll(n),
        }
    }
}

impl GetMaxValue for NumericDice {
    fn get_max_value(&self) -> NumericRoll {
        match self {
            NumericDice::ConstDice(dice) => dice.get_max_value(),
            NumericDice::NumberedDice(dice) => dice.get_max_value(),
            NumericDice::RepeatingDice(dice) => dice.get_max_value(),
        }
    }
}

impl Roll for TextDice {
    type RollResult = Vec<TextRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        match self {
            TextDice::FudgeDice(dice) => dice.roll(n),
            TextDice::ConstDice(dice) => dice.roll(n),
            TextDice::RepeatingDice(dice) => dice.roll(n),
        }
    }
}

pub enum Rolls {
    NumericRolls(Vec<NumericRoll>),
    TextRolls(Vec<TextRoll>),
}
/// Dice that always return the same value
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ConstDice<T: Debug + PartialEq + Eq + Hash> {
    pub(crate) const_value: T,
}

impl<T: Debug + PartialEq + Eq + Hash> ConstDice<T> {
    pub fn new(const_value: T) -> ConstDice<T> {
        ConstDice { const_value }
    }
}

impl<T: Debug + PartialEq + Eq + Hash + Copy> Roll for ConstDice<T> {
    type RollResult = Vec<T>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        iter::repeat(self.const_value).take(n as usize).collect()
    }
}

impl GetMaxValue for ConstDice<NumericRoll> {
    fn get_max_value(&self) -> NumericRoll {
        self.const_value
    }
}

/// Dice that return the same list of values
///
/// Useful for tests
#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RepeatingDice<T: Debug + PartialEq + Eq + Hash> {
    pub(crate) values: Vec<T>,
}

impl<T: Debug + PartialEq + Eq + Hash> RepeatingDice<T> {
    pub fn new(values: Vec<T>) -> Result<RepeatingDice<T>, Error> {
        match values.len() {
            0 => Err(Error::new(ErrorKind::BadDice(String::from(
                "Can't create a repeating dice with an empty values list",
            )))),
            _ => Ok(RepeatingDice { values }),
        }
    }
}

impl<T: Debug + PartialEq + Eq + Hash + Copy> Roll for RepeatingDice<T> {
    type RollResult = Vec<T>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        let mut repeat_values = self.values.clone();
        for _ in 0..(n as usize / self.values.len()) {
            repeat_values.append(&mut self.values.clone());
        }
        repeat_values[0..(n as usize)].to_vec()
    }
}

impl GetMaxValue for RepeatingDice<NumericRoll> {
    fn get_max_value(&self) -> NumericRoll {
        *self.values.iter().max().unwrap_or(&(0 as NumericRoll))
    }
}

#[derive(Debug)]
pub struct NumberedDice {
    sides: NumericRoll,
    rng_ref: RefCell<ThreadRng>,
}

impl NumberedDice {
    pub fn new(sides: NumericRoll) -> NumberedDice {
        NumberedDice {
            sides,
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }
}

impl PartialEq for NumberedDice {
    fn eq(&self, other: &NumberedDice) -> bool {
        self.sides == other.sides
    }
}

impl Eq for NumberedDice {}

impl Roll for NumberedDice {
    type RollResult = Vec<NumericRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1)
            .map(|_| rng.gen_range(1, self.sides + 1))
            .collect()
    }
}

impl GetMaxValue for NumberedDice {
    fn get_max_value(&self) -> NumericRoll {
        self.sides
    }
}

impl Hash for NumberedDice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sides.hash(state);
    }
}

#[derive(Debug)]
pub struct FudgeDice {
    rng_ref: RefCell<ThreadRng>,
}

impl FudgeDice {
    pub fn new() -> FudgeDice {
        FudgeDice {
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }
}

impl Roll for FudgeDice {
    type RollResult = Vec<TextRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1)
            .map(|_| match rng.gen_range(1, 4) {
                1 => '0',
                2 => '+',
                _ => '-',
            })
            .collect()
    }
}

impl PartialEq for FudgeDice {
    fn eq(&self, _: &FudgeDice) -> bool {
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
    use crate::dice::*;

    #[test]
    fn dice_kind_comparison() {
        assert_eq!(
            NumericDice::ConstDice(ConstDice::new(10)),
            NumericDice::ConstDice(ConstDice::new(10))
        );
        assert_ne!(
            NumericDice::ConstDice(ConstDice::new(10)),
            NumericDice::ConstDice(ConstDice::new(20))
        );
        assert_eq!(
            NumericDice::NumberedDice(NumberedDice::new(10)),
            NumericDice::NumberedDice(NumberedDice::new(10))
        );
        assert_ne!(
            NumericDice::NumberedDice(NumberedDice::new(10)),
            NumericDice::NumberedDice(NumberedDice::new(30))
        );
        assert_ne!(
            NumericDice::NumberedDice(NumberedDice::new(10)),
            NumericDice::ConstDice(ConstDice::new(10))
        );
        assert_eq!(FudgeDice::new(), FudgeDice::new());
        assert_eq!(
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10)))
        );
        assert_ne!(
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
            DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(10)))
        );
        assert_ne!(
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
            DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new()))
        );
    }

    #[test]
    fn const_generation() {
        let const_value = 42;
        let roll_number = 5;
        let gen = ConstDice::new(const_value);
        let rolls = gen.roll(roll_number);
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert_eq!(*roll, const_value);
        }
    }

    #[test]
    fn numbered_dice_generation() {
        let dice_sides = 42;
        let roll_number = 5;
        let gen = NumberedDice::new(dice_sides);
        let rolls = gen.roll(roll_number);
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert!(*roll > 0, "Numbered dice generator rolls should be > 0");
            assert!(
                *roll <= dice_sides,
                "Numbered dice generator rolls should be <= to the number of sides on the dice"
            );
        }
    }

    #[test]
    fn repeating_dice() {
        let dice = RepeatingDice::new(vec![1, 2, 3, 4, 5]);
        match dice {
            Err(_) => assert!(false),
            Ok(dice) => {
                assert_eq!(dice.roll(0), vec![]);
                assert_eq!(dice.roll(3), vec![1, 2, 3]);
                assert_eq!(dice.roll(5), vec![1, 2, 3, 4, 5]);
                assert_eq!(
                    dice.roll(15),
                    vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5]
                );
            }
        }
    }

    #[test]
    fn repeating_dice_empty() {
        let empty_value: Vec<NumericRoll> = vec![];
        let dice = RepeatingDice::new(empty_value);
        match dice {
            Err(_) => assert!(true),
            Ok(_) => assert!(false),
        }
    }
}

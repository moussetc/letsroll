use core::fmt::Debug;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;

use crate::errors::Error;

use crate::actions;

pub type DiceNumber = u8;
/// Type of roll result for numbered dice (like D20)
pub type NumericRoll = u16;
// Type of roll result for fudge dice (fate)
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FudgeRoll {
    Plus,
    Minus,
    Blank,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NumericDice {
    ConstDice(NumericRoll),
    NumberedDice(NumericRoll),
    RepeatingDice(Vec<NumericRoll>),
}

impl NumericDice {
    pub fn get_max_value(&self) -> NumericRoll {
        match self {
            NumericDice::ConstDice(const_value) => *const_value,
            NumericDice::NumberedDice(sides) => *sides,
            NumericDice::RepeatingDice(repeating_values) => {
                *repeating_values.iter().max().unwrap_or(&0)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum FudgeDice {
    FudgeDice,
    ConstDice(FudgeRoll),
    RepeatingDice(Vec<FudgeRoll>),
}

#[derive(Debug)]
pub struct Dice {
    rng_ref: RefCell<ThreadRng>,
}

impl Dice {
    pub fn new() -> Dice {
        Dice {
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }

    pub fn roll_numeric_dice(&self, n: DiceNumber, dice: &NumericDice) -> Vec<NumericRoll> {
        match dice {
            NumericDice::ConstDice(const_value) => self.roll_const_dice(n, const_value),
            NumericDice::NumberedDice(sides) => self.roll_numbered_dice(n, sides),
            NumericDice::RepeatingDice(repeating_values) => {
                self.roll_repeating(n, repeating_values)
            }
        }
    }

    pub fn roll_repeating<T: Clone>(&self, n: DiceNumber, repeating_values: &Vec<T>) -> Vec<T> {
        let mut repeat_values = repeating_values.clone();
        for _ in 0..(n as usize / repeating_values.len()) {
            repeat_values.append(&mut repeating_values.clone());
        }
        repeat_values[0..(n as usize)].to_vec()
    }

    pub fn roll_const_dice<T: Clone>(&self, n: DiceNumber, const_value: &T) -> Vec<T> {
        (1..n + 1).map(|_| const_value.clone()).collect()
    }

    pub fn roll_numbered_dice(&self, n: DiceNumber, sides: &NumericRoll) -> Vec<NumericRoll> {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1).map(|_| rng.gen_range(1, sides + 1)).collect()
    }

    pub fn roll_fudgey_dice(&self, n: DiceNumber, dice: &FudgeDice) -> Vec<FudgeRoll> {
        match dice {
            FudgeDice::ConstDice(const_value) => self.roll_const_dice(n, const_value),
            FudgeDice::FudgeDice => self.roll_fudge_dice(n),
            FudgeDice::RepeatingDice(repeating_values) => self.roll_repeating(n, repeating_values),
        }
    }

    pub fn roll_fudge_dice(&self, n: DiceNumber) -> Vec<FudgeRoll> {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1)
            .map(|_| match rng.gen_range(1, 4) {
                1 => FudgeRoll::Blank,
                2 => FudgeRoll::Plus,
                _ => FudgeRoll::Minus,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DiceRequest<T: Clone> {
    pub(crate) number: DiceNumber,
    pub(crate) dice: T,
}

impl<T: Clone> DiceRequest<T> {
    pub fn new(number: DiceNumber, dice: T) -> DiceRequest<T> {
        DiceRequest { number, dice }
    }
}

#[derive(Debug)]
pub struct RollResults<T: Debug, V: Debug + Clone> {
    pub(crate) dice_request: DiceRequest<V>,
    pub(crate) description: String,
    pub(crate) rolls: Vec<T>,
}

impl RollResults<NumericRoll, NumericDice> {
    pub fn new(
        dice_request: DiceRequest<NumericDice>,
        dice: &Dice,
    ) -> RollResults<NumericRoll, NumericDice> {
        RollResults {
            description: dice_request.to_string(),
            rolls: dice.roll_numeric_dice(dice_request.number, &dice_request.dice),
            dice_request,
        }
    }
}

impl RollResults<FudgeRoll, FudgeDice> {
    pub fn new(
        dice_request: DiceRequest<FudgeDice>,
        dice: &Dice,
    ) -> RollResults<FudgeRoll, FudgeDice> {
        RollResults {
            description: dice_request.to_string(),
            rolls: dice.roll_fudgey_dice(dice_request.number, &dice_request.dice),
            dice_request,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::*;

    // TODO if test passes, that's because of how rust work so the test should be deleted!
    #[test]
    fn dice_kind_comparison() {
        assert_eq!(NumericDice::ConstDice(10), NumericDice::ConstDice(10));
        assert_ne!(NumericDice::ConstDice(10), NumericDice::ConstDice(20));
        assert_eq!(NumericDice::NumberedDice(10), NumericDice::NumberedDice(10));
        assert_ne!(NumericDice::NumberedDice(10), NumericDice::NumberedDice(30));
        assert_ne!(NumericDice::NumberedDice(10), NumericDice::ConstDice(10));
    }

    #[test]
    fn const_generation() {
        let dice = Dice::new();
        let const_value = 42;
        let roll_number = 5;
        let rolls = dice.roll_numeric_dice(roll_number, &NumericDice::ConstDice(const_value));
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert_eq!(*roll, const_value);
        }

        let const_value = FudgeRoll::Blank;
        let roll_number = 2;
        let rolls = dice.roll_fudgey_dice(roll_number, &FudgeDice::ConstDice(const_value));
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert_eq!(*roll, const_value);
        }
    }

    #[test]
    fn numbered_dice_generation() {
        let dice = Dice::new();
        let dice_sides = 42;
        let roll_number = 5;
        let rolls = dice.roll_numeric_dice(roll_number, &NumericDice::NumberedDice(dice_sides));
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
        let dice = Dice::new();
        let repeating_values = vec![1, 2, 3, 4, 5];

        assert_eq!(
            dice.roll_numeric_dice(0, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![]
        );
        assert_eq!(
            dice.roll_numeric_dice(3, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![1, 2, 3]
        );
        assert_eq!(
            dice.roll_numeric_dice(5, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            dice.roll_numeric_dice(15, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5]
        );
    }
}

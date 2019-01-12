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
// Type of roll result for fudge dice (fate)
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FudgeRoll {
    Plus,
    Minus,
    Blank,
}

#[cfg(test)]
mod tests {
    use crate::dice::*;

    // #[test]
    // fn dice_kind_comparison() {
    //     assert_eq!(
    //         NumericDice::ConstDice(ConstDice::new(10)),
    //         NumericDice::ConstDice(ConstDice::new(10))
    //     );
    //     assert_ne!(
    //         NumericDice::ConstDice(ConstDice::new(10)),
    //         NumericDice::ConstDice(ConstDice::new(20))
    //     );
    //     assert_eq!(
    //         NumericDice::NumberedDice(NumberedDice::new(10)),
    //         NumericDice::NumberedDice(NumberedDice::new(10))
    //     );
    //     assert_ne!(
    //         NumericDice::NumberedDice(NumberedDice::new(10)),
    //         NumericDice::NumberedDice(NumberedDice::new(30))
    //     );
    //     assert_ne!(
    //         NumericDice::NumberedDice(NumberedDice::new(10)),
    //         NumericDice::ConstDice(ConstDice::new(10))
    //     );
    //     assert_eq!(FudgeDice::new(), FudgeDice::new());
    //     assert_eq!(
    //         DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
    //         DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10)))
    //     );
    //     assert_ne!(
    //         DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
    //         DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(10)))
    //     );
    //     assert_ne!(
    //         DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
    //         DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new()))
    //     );
    // }

    // #[test]
    // fn const_generation() {
    //     let const_value = 42;
    //     let roll_number = 5;
    //     let gen = ConstDice::new(const_value);
    //     let rolls = gen.roll(roll_number);
    //     assert_eq!(rolls.len(), roll_number as usize);
    //     for roll in rolls.iter() {
    //         assert_eq!(*roll, const_value);
    //     }
    // }

    // #[test]
    // fn numbered_dice_generation() {
    //     let dice_sides = 42;
    //     let roll_number = 5;
    //     let gen = NumberedDice::new(dice_sides);
    //     let rolls = gen.roll(roll_number);
    //     assert_eq!(rolls.len(), roll_number as usize);
    //     for roll in rolls.iter() {
    //         assert!(*roll > 0, "Numbered dice generator rolls should be > 0");
    //         assert!(
    //             *roll <= dice_sides,
    //             "Numbered dice generator rolls should be <= to the number of sides on the dice"
    //         );
    //     }
    // }

    // #[test]
    // fn repeating_dice() {
    //     let dice = RepeatingDice::new(vec![1, 2, 3, 4, 5]);
    //     match dice {
    //         Err(_) => assert!(false),
    //         Ok(dice) => {
    //             assert_eq!(dice.roll(0), vec![]);
    //             assert_eq!(dice.roll(3), vec![1, 2, 3]);
    //             assert_eq!(dice.roll(5), vec![1, 2, 3, 4, 5]);
    //             assert_eq!(
    //                 dice.roll(15),
    //                 vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5]
    //             );
    //         }
    //     }
    // }

    // #[test]
    // fn repeating_dice_empty() {
    //     let empty_value: Vec<NumericRoll> = vec![];
    //     let dice = RepeatingDice::new(empty_value);
    //     match dice {
    //         Err(_) => assert!(true),
    //         Ok(_) => assert!(false),
    //     }
    // }
}

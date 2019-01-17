use crate::dice::*;
use core::fmt::Debug;
use std::fmt::{self, Display};

impl Display for FudgeRoll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FudgeRoll::Blank => '0',
                FudgeRoll::Plus => '+',
                FudgeRoll::Minus => '-',
            }
        )
    }
}

impl fmt::Display for NumericDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NumericDice::ConstDice(const_value) => const_value.to_string(),
                NumericDice::NumberedDice(sides) => format!("D{}", sides),
                NumericDice::RepeatingDice(repeat_values) => format!(
                    "[{}...]",
                    repeat_values
                        .iter()
                        .map(|val| val.to_string() + ",")
                        .collect::<String>()
                ),
                NumericDice::AggregationResult => String::from("AggregatedValue"),
            }
        )
    }
}

impl fmt::Display for FudgeDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FudgeDice::ConstDice(const_value) => const_value.to_string(),
                FudgeDice::FudgeDice => String::from("F"),
                FudgeDice::RepeatingDice(repeat_values) => format!(
                    "[{}...]",
                    repeat_values
                        .iter()
                        .map(|val| val.to_string() + ",")
                        .collect::<String>()
                ),
            }
        )
    }
}

impl<T: Display + Clone> fmt::Display for RollRequest<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.dice.to_string())
    }
}

impl<T: Display + Debug, V: Clone + Debug> fmt::Display for Rolls<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.description,
            self.rolls
                .iter()
                .map(|roll| roll.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

#[cfg(test)]
mod tests {

    // use crate::dice::*;
    // use crate::*;

    // #[test]
    // fn numeric_roll_to_string() {
    //     assert_eq!(ConstDice::new(20).roll(1)[0].to_string(), "20");
    // }

    // #[test]
    // fn fudge_roll_to_string() {
    //     assert_eq!(ConstDice::new(FudgeRoll::Blank).roll(1)[0].to_string(), "0");
    //     assert_eq!(ConstDice::new(FudgeRoll::Minus).roll(1)[0].to_string(), "-");
    //     assert_eq!(ConstDice::new(FudgeRoll::Plus).roll(1)[0].to_string(), "+");
    // }

    // #[test]
    // fn numbered_dice_to_string() {
    //     assert_eq!(NumberedDice::new(20).to_string(), "D20");
    // }

    // #[test]
    // fn fudge_dice_to_string() {
    //     assert_eq!(FudgeDice::new().to_string(), "F");
    // }

    // #[test]
    // fn const_dice_to_string() {
    //     assert_eq!(ConstDice::new(42).to_string(), "+42");
    // }

    // #[test]
    // fn repeating_dice_to_string() {
    //     assert_eq!(
    //         RepeatingDice::new(vec![1, 2, 3]).unwrap().to_string(),
    //         "[1,2,3,...]"
    //     );
    // }

    // #[test]
    // fn dice_request_to_string() {
    //     assert_eq!(
    //         RollRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 5)
    //             .to_string(),
    //         "5F"
    //     );
    //     assert_eq!(
    //         RollRequest::new(
    //             DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(6))),
    //             1
    //         )
    //         .to_string(),
    //         "1D6"
    //     );
    // }

}

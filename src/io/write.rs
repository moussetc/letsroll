use crate::dice::*;
use crate::{DiceRequest, RollRequest};
use std::fmt::{self, Debug, Display};
use std::hash::Hash;

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

impl fmt::Display for Rolls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rolls::NumericRolls(rolls) => rolls
                    .iter()
                    .map(|roll| roll.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                Rolls::FudgeRolls(rolls) => rolls
                    .iter()
                    .map(|roll| roll.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            }
        )
    }
}

impl<T: Debug + PartialEq + Eq + Hash + Display> fmt::Display for ConstDice<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("+{}", self.const_value),)
    }
}

impl<T: Debug + PartialEq + Eq + Hash + Display> fmt::Display for RepeatingDice<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "[{}...]",
                self.values
                    .iter()
                    .map(|val| val.to_string() + ",")
                    .collect::<String>()
            )
        )
    }
}

impl fmt::Display for NumberedDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("D{}", self.get_max_value()))
    }
}

impl fmt::Display for FudgeDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "F")
    }
}

impl fmt::Display for NumericDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NumericDice::ConstDice(dice) => dice.to_string(),
                NumericDice::NumberedDice(dice) => dice.to_string(),
                NumericDice::RepeatingDice(dice) => dice.to_string(),
            }
        )
    }
}

impl fmt::Display for TextDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TextDice::FudgeDice(dice) => dice.to_string(),
                TextDice::ConstDice(dice) => dice.to_string(),
                TextDice::RepeatingDice(dice) => dice.to_string(),
            }
        )
    }
}

impl fmt::Display for DiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DiceKind::NumericKind(num_dice) => num_dice.to_string(),
                DiceKind::TextKind(text_dice) => text_dice.to_string(),
            }
        )
    }
}

impl fmt::Display for DiceRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.kind,)
    }
}

impl fmt::Display for RollRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.results()
                .iter()
                .map(|keyval| format!("{}\t: {}", keyval.0, keyval.1))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::dice::*;
    use crate::*;

    #[test]
    fn numeric_roll_to_string() {
        assert_eq!(ConstDice::new(20).roll(1)[0].to_string(), "20");
    }

    #[test]
    fn fudge_roll_to_string() {
        assert_eq!(ConstDice::new(FudgeRoll::Blank).roll(1)[0].to_string(), "0");
        assert_eq!(ConstDice::new(FudgeRoll::Minus).roll(1)[0].to_string(), "-");
        assert_eq!(ConstDice::new(FudgeRoll::Plus).roll(1)[0].to_string(), "+");
    }

    #[test]
    fn numbered_dice_to_string() {
        assert_eq!(NumberedDice::new(20).to_string(), "D20");
    }

    #[test]
    fn fudge_dice_to_string() {
        assert_eq!(FudgeDice::new().to_string(), "F");
    }

    #[test]
    fn const_dice_to_string() {
        assert_eq!(ConstDice::new(42).to_string(), "+42");
    }

    #[test]
    fn repeating_dice_to_string() {
        assert_eq!(
            RepeatingDice::new(vec![1, 2, 3]).unwrap().to_string(),
            "[1,2,3,...]"
        );
    }

    #[test]
    fn dice_request_to_string() {
        assert_eq!(
            DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 5)
                .to_string(),
            "5F"
        );
        assert_eq!(
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(6))),
                1
            )
            .to_string(),
            "1D6"
        );
    }

}

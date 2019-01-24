use crate::dice::*;
use crate::MultiTypeSession;
use crate::TypedRollSession;
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
                NumericDice::ConstDice(const_value) => format!("+{}", const_value.to_string()),
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

impl<T: DiceBounds> fmt::Display for RollRequest<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = match &self.id {
            Some(ref id) => format!("{}: ", id),
            None => String::from(""),
        };
        write!(f, "{}{}{}", id, self.number, self.dice.to_string())
    }
}

impl<T: RollBounds, V: DiceBounds> fmt::Display for Rolls<T, V> {
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

impl<T: RollBounds, V: DiceBounds> ToString for TypedRollSession<T, V> {
    fn to_string(&self) -> String {
        self.rolls
            .iter()
            .map(|roll| roll.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl ToString for MultiTypeSession {
    fn to_string(&self) -> String {
        let mut subresults: Vec<String> = vec![];
        if let Some(session) = &self.numeric_session {
            subresults.push(session.to_string());
        }
        if let Some(session) = &self.fudge_session {
            subresults.push(session.to_string());
        }
        subresults.join("\n")
    }
}

#[cfg(test)]
mod tests {

    use crate::actions::Action;
    use crate::dice::*;

    #[test]
    fn numeric_roll_to_string() {
        let dice = DiceGenerator::new();
        assert_eq!(
            dice.roll(1, &NumericDice::ConstDice(20))[0].to_string(),
            "20"
        );
    }

    #[test]
    fn fudge_roll_to_string() {
        let dice = DiceGenerator::new();
        assert_eq!(
            dice.roll(1, &FudgeDice::ConstDice(FudgeRoll::Blank))[0].to_string(),
            "0"
        );
        assert_eq!(
            dice.roll(1, &FudgeDice::ConstDice(FudgeRoll::Minus))[0].to_string(),
            "-"
        );
        assert_eq!(
            dice.roll(1, &FudgeDice::ConstDice(FudgeRoll::Plus))[0].to_string(),
            "+"
        );
    }

    #[test]
    fn numbered_dice_to_string() {
        assert_eq!(NumericDice::NumberedDice(20).to_string(), "D20");
    }

    #[test]
    fn fudge_dice_to_string() {
        assert_eq!(FudgeDice::FudgeDice.to_string(), "F");
    }

    #[test]
    fn const_dice_to_string() {
        assert_eq!(NumericDice::ConstDice(42).to_string(), "+42");
    }

    #[test]
    fn repeating_dice_to_string() {
        assert_eq!(
            NumericDice::RepeatingDice(vec![1, 2, 3]).to_string(),
            "[1,2,3,...]"
        );
    }

    #[test]
    fn dice_request_to_string() {
        assert_eq!(RollRequest::new(5, FudgeDice::FudgeDice).to_string(), "5F");
        assert_eq!(
            RollRequest::new(1, NumericDice::NumberedDice(6)).to_string(),
            "1D6"
        );
        assert_eq!(
            RollRequest::new(10, NumericDice::NumberedDice(12))
                .add_id(Some(String::from("FIRE")))
                .add_action(Action::KeepBest(1))
                .to_string(),
            "FIRE: 10D12"
        );
    }

}

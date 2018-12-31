pub mod actions;
pub mod dice;
pub mod errors;

use crate::actions::{Action, CountValues, FlipFlop, Identity, MultiplyBy, Reroll, Sum};
use crate::dice::*;
use crate::errors::{Error, ErrorKind};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DiceRequest {
    kind: DiceKind,
    number: DiceNumber,
}
impl DiceRequest {
    pub fn new(kind: DiceKind, number: DiceNumber) -> DiceRequest {
        DiceRequest { kind, number }
    }
}

impl FromStr for DiceRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("F") {
            let number_from_str = s[0..s.len() - 1].parse::<u8>()?;
            return Ok(DiceRequest {
                kind: DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())),
                number: number_from_str,
            });
        }

        let parts: Vec<&str> = s.trim().split('D').collect();

        // Try to read a numbered dice request (no other dice implement yet)
        match parts.len() {
            1 | 2 => {
                let mut number_fromstr = parts[0].parse::<u8>()?;
                let sides_fromstr = parts[parts.len() - 1].parse::<u16>()?;
                // the number of dice to roll is optional and efaults to 1
                if parts.len() == 1 {
                    number_fromstr = 1;
                }
                Ok(DiceRequest {
                    number: number_fromstr,
                    kind: DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(
                        sides_fromstr,
                    ))),
                })
            }
            _ => Err(Error::new(errors::ErrorKind::ParseDice(format!(
                "Expected no more than two parts but found \"{}\" ",
                parts.len()
            )))),
        }
    }
}

impl fmt::Display for DiceRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.kind,)
    }
}

pub struct RollRequest {
    rolls: HashMap<DiceRequest, Rolls>,
}

impl RollRequest {
    pub fn new(dice_requests: Vec<DiceRequest>) -> RollRequest {
        let mut request = RollRequest {
            rolls: HashMap::new(),
        };
        let requests = dice_requests;
        {
            for mut dice in requests.into_iter() {
                // Initial rolls
                let rolls = match dice.kind {
                    DiceKind::NumericKind(ref mut num_dice) => {
                        Rolls::NumericRolls(num_dice.roll(dice.number))
                    }
                    DiceKind::TextKind(ref mut text_dice) => {
                        Rolls::TextRolls(text_dice.roll(dice.number))
                    }
                };
                request.rolls.insert(dice, rolls);
            }
        }
        request
    }

    pub fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        for (dice, rolls) in self.rolls.iter_mut() {
            *rolls = match rolls {
                Rolls::NumericRolls(num_rolls) => match &dice.kind {
                    DiceKind::NumericKind(num_dice) => match &num_dice {
                        NumericDice::NumberedDice(_) | NumericDice::Mock(_) => {
                            match RollRequest::add_step_numeric_input(num_dice, num_rolls, &action)
                            {
                                Ok(new_rolls) => new_rolls,
                                Err(error) => Err(error)?,
                            }
                        }
                    },
                    _ => {
                        return Err(Error::incompatible(
                            &action.to_string(),
                            &String::from("numeric roll"),
                        ));
                    }
                },
                Rolls::TextRolls(text_rolls) => match &dice.kind {
                    DiceKind::TextKind(text_dice) => match &text_dice {
                        TextDice::FudgeDice(_) => {
                            match RollRequest::add_step_text_input(text_dice, text_rolls, &action) {
                                Ok(new_rolls) => new_rolls,
                                Err(error) => Err(error)?,
                            }
                        }
                    },
                    _ => {
                        return Err(Error::incompatible(
                            &action.to_string(),
                            &String::from("numeric roll"),
                        ));
                    }
                },
            };

            // TODO : apply rerolls
        }
        Ok(())
    }

    fn add_step_text_input(
        dice: &TextDice,
        text_rolls: &mut Vec<TextRoll>,
        action: &actions::Action,
    ) -> Result<(Rolls), Error> {
        Ok(match action {
            Action::Identity => Rolls::TextRolls(text_rolls.do_nothing()),
            Action::CountValues => Rolls::NumericRolls(text_rolls.count()),
            Action::RerollText(value_to_reroll) => match dice {
                TextDice::FudgeDice(ref d) => {
                    Rolls::TextRolls(text_rolls.reroll(d, &value_to_reroll))
                }
            },
            _ => {
                return Err(Error::new(ErrorKind::IncompatibleAction(format!(
                    "Action {:?} not supported by roll type {:?}",
                    action,
                    String::from("text roll")
                ))))
            }
        })
    }

    fn add_step_numeric_input(
        dice: &NumericDice,
        num_rolls: &mut Vec<NumericRoll>,
        action: &actions::Action,
    ) -> Result<(Rolls), Error> {
        Ok(Rolls::NumericRolls(match action {
            Action::Identity => num_rolls.do_nothing(),
            Action::CountValues => num_rolls.count(),
            Action::FlipFlop => match dice {
                NumericDice::NumberedDice(ref d) => num_rolls.flip(d),
                NumericDice::Mock(ref d) => num_rolls.flip(d),
            },
            Action::RerollNumeric(value_to_reroll) => match dice {
                NumericDice::Mock(d) => num_rolls.reroll(d, &value_to_reroll),
                NumericDice::NumberedDice(ref d) => num_rolls.reroll(d, &value_to_reroll),
            },
            Action::MultiplyBy(factor) => num_rolls.multiply(*factor),
            Action::Sum => num_rolls.sum(),
            _ => {
                return Err(Error::new(ErrorKind::IncompatibleAction(format!(
                    "Action {:?} not supported by roll type {:?}",
                    action,
                    String::from("numeric roll")
                ))))
            }
        }))
    }

    pub fn results(&self) -> &HashMap<DiceRequest, Rolls> {
        &self.rolls
    }
}

impl fmt::Display for RollRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.results()
                .iter()
                .map(|keyval| format!("{} : {}|", keyval.0, keyval.1))
                .collect::<Vec<String>>()
                .join(" "),
        )
    }
}

impl FromStr for RollRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<DiceRequest> = s
            .trim()
            .split(' ')
            .map(|part| DiceRequest::from_str(part))
            .collect::<Result<Vec<DiceRequest>, Error>>()?;

        Ok(RollRequest::new(parts))
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Action;
    use crate::dice::{DiceKind, FudgeDice, Mock, NumberedDice, NumericDice, TextDice};
    use crate::DiceRequest;

    #[test]
    fn request_identity() {
        test_action_implemented_for_types(Action::Identity, true, true);
    }

    #[test]
    fn request_count_values() {
        test_action_implemented_for_types(Action::CountValues, true, true);
    }

    #[test]
    fn request_reroll_numeric() {
        test_action_implemented_for_types(Action::RerollNumeric(1), true, false);
    }

    #[test]
    fn request_reroll_text() {
        test_action_implemented_for_types(Action::RerollText(' '), false, true);
    }

    #[test]
    fn request_sum() {
        test_action_implemented_for_types(Action::Sum, true, false);
    }

    #[test]
    fn request_multiply_by() {
        test_action_implemented_for_types(Action::MultiplyBy(42), true, false);
    }

    #[test]
    fn request_flipflop() {
        test_action_implemented_for_types(Action::FlipFlop, true, false);
    }

    /// Test the compatibility between actions and roll types
    fn test_action_implemented_for_types(
        action: Action,
        test_num_types: bool,
        test_text_types: bool,
    ) {
        let dice_number = 5;
        let dice_val = 15;

        assert!(
            test_num_types || test_text_types,
            "This test function should be called with at least one type enabled"
        );

        // Should be implemented for all dice types
        let mut dice_requests = vec![];
        if test_num_types {
            dice_requests.push(DiceRequest::new(
                DiceKind::NumericKind(NumericDice::Mock(Mock::new(dice_val))),
                dice_number,
            ));
            dice_requests.push(DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(dice_val))),
                dice_number,
            ));
        }
        if test_text_types {
            dice_requests.push(DiceRequest::new(
                DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())),
                dice_number,
            ));
        }

        let dice_requests_len = dice_requests.len();

        let mut request = crate::RollRequest::new(dice_requests);
        assert_eq!(Ok(()), request.add_step(action));
        let output = request.results();

        assert_eq!(output.len(), dice_requests_len);
    }

}

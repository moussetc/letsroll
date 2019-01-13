pub mod actions;
pub mod dice;
pub mod errors;
pub mod io;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub use crate::actions::Action;
use crate::actions::*;
use crate::dice::*;
use crate::errors::Error;
use core::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
pub struct TypedRollSession<T: Debug, V: Debug + Clone> {
    pub rolls: Vec<Rolls<T, V>>,
    dice: Dice,
}

pub type NumericSession = TypedRollSession<NumericRoll, NumericDice>;
pub type FudgeSession = TypedRollSession<FudgeRoll, FudgeDice>;

impl NumericSession {
    pub fn new(dice_requests: Vec<NumericRollRequest>) -> NumericSession {
        let dice = Dice::new();
        TypedRollSession {
            rolls: dice_requests
                .into_iter()
                .map(|dice_request| NumericRolls::new(dice_request, &dice))
                .collect(),
            dice,
        }
    }
}

impl FudgeSession {
    pub fn new(dice_requests: Vec<FudgeRollRequest>) -> FudgeSession {
        let dice = Dice::new();
        TypedRollSession {
            rolls: dice_requests
                .into_iter()
                .map(|dice_request| FudgeRolls::new(dice_request, &dice))
                .collect(),
            dice,
        }
    }
}

pub trait Session: Debug {
    fn to_string(&self) -> String;

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error>;

    fn write_results_to_file(&self, filepath: &str) -> std::io::Result<()> {
        let path = Path::new(filepath);

        let mut file = File::create(&path)?;
        file.write_all(self.to_string().as_bytes())
    }
}

impl Session for NumericSession {
    fn to_string(&self) -> String {
        self.rolls
            .iter()
            .map(|roll| roll.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        match action {
            Action::Sum => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.sum();
                }
            }
            Action::Total => self.rolls = vec![self.rolls.total()],
            Action::MultiplyBy(factor) => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.multiply(factor);
                }
            }
            Action::Explode(explosion_value) => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.explode(&self.dice, &explosion_value);
                }
            }
            Action::FlipFlop => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.flip();
                }
            }
            Action::RerollNumeric(values_to_reroll) => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.reroll(&self.dice, &values_to_reroll);
                }
            }
            Action::RerollFudge(_) | Action::ExplodeFudge(_) => {
                return Err(Error::incompatible(
                    &action.to_string(),
                    &String::from("numeric roll"),
                ));
            }
        }
        Ok(())
    }
}

impl Session for FudgeSession {
    fn to_string(&self) -> String {
        self.rolls
            .iter()
            .map(|roll| roll.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        match action {
            Action::ExplodeFudge(explosion_value) => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.explode(&self.dice, &explosion_value);
                }
            }
            Action::RerollFudge(values_to_reroll) => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.reroll(&self.dice, &values_to_reroll);
                }
            }
            Action::Sum
            | Action::Total
            | Action::MultiplyBy(_)
            | Action::FlipFlop
            | Action::RerollNumeric(_)
            | Action::Explode(_) => {
                return Err(Error::incompatible(
                    &action.to_string(),
                    &String::from("fudge roll"),
                ));
            }
        }
        Ok(())
    }
}

pub trait AggregatableSession: Debug {
    fn aggregate(self, action: &Aggregation) -> NumericSession;
}

impl AggregatableSession for NumericSession {
    fn aggregate(self, action: &Aggregation) -> NumericSession {
        // TODO other kind of aggregation ?
        match action {
            Aggregation::CountValues => self.count(),
        }
    }
}

impl AggregatableSession for FudgeSession {
    fn aggregate(self, action: &Aggregation) -> NumericSession {
        match action {
            Aggregation::CountValues => self.count(),
        }
    }
}

#[derive(Debug)]
pub struct MultiTypeSession {
    numeric_session: Option<NumericSession>,
    fudge_session: Option<FudgeSession>,
}

impl Session for MultiTypeSession {
    fn to_string(&self) -> String {
        let mut subresults: Vec<String> = vec![];
        match &self.numeric_session {
            Some(session) => subresults.push(session.to_string()),
            None => (),
        };
        match &self.fudge_session {
            Some(session) => subresults.push(session.to_string()),
            None => (),
        };
        subresults.join("\n")
    }

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        match &mut self.numeric_session {
            Some(ref mut session) => session.add_step(action.clone())?,
            None => (),
        };
        match &mut self.fudge_session {
            Some(ref mut session) => session.add_step(action.clone())?,
            None => (),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use crate::actions::Action;
    // use crate::dice::{
    //     ConstDice, DiceKind, FudgeDice, FudgeRoll, NumberedDice, NumericDice, TextDice,
    // };
    // use crate::RollRequest;

    // #[test]
    // fn request_count_values() {
    //     test_action_implemented_for_types(Action::CountValues, true, true);
    // }

    // #[test]
    // fn request_reroll_numeric() {
    //     test_action_implemented_for_types(Action::RerollNumeric(1), true, false);
    // }

    // #[test]
    // fn request_reroll_text() {
    //     test_action_implemented_for_types(Action::RerollFudge(FudgeRoll::Blank), false, true);
    // }

    // #[test]
    // fn request_sum() {
    //     test_action_implemented_for_types(Action::Sum, true, false);
    // }

    // #[test]
    // fn request_multiply_by() {
    //     test_action_implemented_for_types(Action::MultiplyBy(42), true, false);
    // }

    // #[test]
    // fn request_flipflop() {
    //     test_action_implemented_for_types(Action::FlipFlop, true, false);
    // }

    // /// Test the compatibility between actions and roll types
    // fn test_action_implemented_for_types(
    //     action: Action,
    //     test_num_types: bool,
    //     test_text_types: bool,
    // ) {
    //     let dice_number = 5;
    //     let dice_val = 15;

    //     assert!(
    //         test_num_types || test_text_types,
    //         "This test function should be called with at least one type enabled"
    //     );

    //     // Should be implemented for all dice types
    //     let mut dice_requests = vec![];
    //     if test_num_types {
    //         dice_requests.push(RollRequest::new(
    //             DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(dice_val))),
    //             dice_number,
    //         ));
    //         dice_requests.push(RollRequest::new(
    //             DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(dice_val))),
    //             dice_number,
    //         ));
    //     }
    //     if test_text_types {
    //         dice_requests.push(RollRequest::new(
    //             DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())),
    //             dice_number,
    //         ));
    //     }

    //     let dice_requests_len = dice_requests.len();

    //     let mut request = crate::RollRequest::new(dice_requests);
    //     assert_eq!(Ok(()), request.add_step(action));
    //     let output = request.dice_rolls;

    //     assert_eq!(output.len(), dice_requests_len);
    // }

}

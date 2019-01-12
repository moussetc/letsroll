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
use crate::errors::{Error, ErrorKind};
use core::fmt::Debug;

#[derive(Debug)]
pub struct TypedRollSession<T: Debug, V: Debug + Clone> {
    rolls: Vec<Rolls<T, V>>,
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
    fn get_results(&self) -> String;
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error>;
}

impl Session for NumericSession {
    fn get_results(&self) -> String {
        self.rolls.iter().map(|roll| roll.to_string()).collect()
    }
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        unimplemented!();
    }
}

impl Session for FudgeSession {
    fn get_results(&self) -> String {
        self.rolls.iter().map(|roll| roll.to_string()).collect()
    }

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct FullRollSession {
    subsessions: Vec<Box<dyn Session>>,
}

impl FullRollSession {
    pub fn new(subsessions: Vec<Box<dyn Session>>) -> FullRollSession {
        FullRollSession { subsessions }
    }
}

impl Session for FullRollSession {
    fn get_results(&self) -> String {
        self.subsessions
            .iter()
            .map(|session| session.get_results())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        unimplemented!();
    }
}

// pub fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
//     // Handle actions that affect all dice first
//     match &action {
//         Action::Total => {
//             // Compute total
//             let total = self.dice_rolls.total(&self.dice_requests)?;
//             let dice = DiceKind::Aggregate(AggregatedDice {
//                 description: String::from("TOTAL"),
//             });
//             self.dice_requests.clear();
//             self.dice_requests.push(RollRequest::new(dice, 1));
//             self.dice_rolls.clear();
//             self.dice_rolls.push(total);
//             return Ok(());
//         }
//         _ => (),
//     }
//     //TODO instead of using indexes to know what dice to use for reroll, define a "dice ID" (later: will be a string, useable by users!)
//     for (dice_index, rolls) in self.dice_rolls.iter_mut().enumerate() {
//         let dice = self
//             .dice_requests
//             .get(dice_index)
//             .expect("not supposed to happen! did an agregate screw everything up?");
//         *rolls = match rolls {
//             Rolls::NumericRolls(num_rolls) => match &dice.kind {
//                 DiceKind::NumericKind(num_dice) => {
//                     match RollRequest::add_step_numeric_input(num_dice, num_rolls, &action) {
//                         Ok(new_rolls) => new_rolls,
//                         Err(error) => Err(error)?,
//                     }
//                 }
//                 _ => {
//                     return Err(Error::incompatible(
//                         &action.to_string(),
//                         &String::from("numeric roll"),
//                     ));
//                 }
//             },
//             Rolls::FudgeRolls(text_rolls) => match &dice.kind {
//                 DiceKind::TextKind(text_dice) => {
//                     match RollRequest::add_step_text_input(text_dice, text_rolls, &action) {
//                         Ok(new_rolls) => new_rolls,
//                         Err(error) => Err(error)?,
//                     }
//                 }
//                 _ => {
//                     return Err(Error::incompatible(
//                         &action.to_string(),
//                         &String::from("text roll"),
//                     ));
//                 }
//             },
//             Rolls::Aggregation(_) => return Err(Error::new(ErrorKind::IncompatibleAction(
//                 String::from("No action can be applied to an aggregated value (eg. the result of a total sum)"))))
//         };
//     }
//     Ok(())
// }

// fn add_step_text_input(
//     dice: &TextDice,
//     text_rolls: &mut Vec<FudgeRoll>,
//     action: &actions::Action,
// ) -> Result<(Rolls), Error> {
//     Ok(match action {
//         Action::CountValues => Rolls::NumericRolls(text_rolls.count()),
//         Action::RerollFudge(value_to_reroll) => match dice {
//             TextDice::ConstDice(text_dice) => {
//                 Rolls::FudgeRolls(text_rolls.reroll(text_dice, &value_to_reroll))
//             }
//             TextDice::FudgeDice(text_dice) => {
//                 Rolls::FudgeRolls(text_rolls.reroll(text_dice, &value_to_reroll))
//             }
//             TextDice::RepeatingDice(text_dice) => {
//                 Rolls::FudgeRolls(text_rolls.reroll(text_dice, &value_to_reroll))
//             }
//         },
//         _ => {
//             return Err(Error::new(ErrorKind::IncompatibleAction(format!(
//                 "Action {:?} not supported by roll type {:?}",
//                 action,
//                 String::from("text roll")
//             ))))
//         }
//     })
// }

// fn add_step_numeric_input(
//     dice: &NumericDice,
//     num_rolls: &mut Vec<NumericRoll>,
//     action: &actions::Action,
// ) -> Result<(Rolls), Error> {
//     Ok(Rolls::NumericRolls(match action {
//         Action::CountValues => num_rolls.count(),
//         Action::FlipFlop => match dice {
//             NumericDice::ConstDice(dice) => num_rolls.flip(dice),
//             NumericDice::NumberedDice(dice) => num_rolls.flip(dice),
//             NumericDice::RepeatingDice(dice) => num_rolls.flip(dice),
//         },
//         Action::RerollNumeric(value_to_reroll) => match dice {
//             NumericDice::ConstDice(dice) => num_rolls.reroll(dice, &value_to_reroll),
//             NumericDice::NumberedDice(dice) => num_rolls.reroll(dice, &value_to_reroll),
//             NumericDice::RepeatingDice(dice) => num_rolls.reroll(dice, &value_to_reroll),
//         },
//         Action::Explode(explosion_value) => match dice {
//             NumericDice::ConstDice(dice) => num_rolls.explode(dice, &explosion_value),
//             NumericDice::NumberedDice(dice) => num_rolls.explode(dice, &explosion_value),
//             NumericDice::RepeatingDice(dice) => num_rolls.explode(dice, &explosion_value),
//         },
//         Action::MultiplyBy(factor) => num_rolls.multiply(*factor),
//         Action::Sum => num_rolls.sum(),
//         _ => {
//             return Err(Error::new(ErrorKind::IncompatibleAction(format!(
//                 "Action {:?} not supported by roll type {:?}",
//                 action,
//                 String::from("numeric roll")
//             ))))
//         }
//     }))
// }

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

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
use core::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// TODO rename the damn struct when brain is working again
#[derive(Debug)]
pub struct RollAndActionsRequest<T: Clone> {
    roll_request: RollRequest<T>,
    pub actions: Vec<Action>,
}

pub type NumericRollAndActionRequest = RollAndActionsRequest<NumericDice>;
pub type FudgeRollAndActionRequest = RollAndActionsRequest<FudgeDice>;

impl<T: Clone> RollAndActionsRequest<T> {
    pub fn new(roll_request: RollRequest<T>, actions: Vec<Action>) -> RollAndActionsRequest<T> {
        RollAndActionsRequest {
            roll_request,
            actions,
        }
    }
}

impl<V: Debug + Clone + Display> RollAndActionsRequest<V> {
    pub fn roll<T: Clone + Debug + Display>(self, dice: &Roll<T, V>) -> Result<Rolls<T, V>, Error>
    where
        Rolls<T, V>: Apply<T, V>,
    {
        let mut rolls = Rolls::<T, V>::new(self.roll_request, dice);
        for action in self.actions.iter() {
            rolls = Apply::<T, V>::apply(&rolls, action, dice)?;
        }
        Ok(rolls)
    }
}

#[derive(Debug)]
pub struct TypedRollSession<T: Debug, V: Debug + Clone> {
    pub rolls: Vec<Rolls<T, V>>,
    dice: DiceGenerator,
}

pub type NumericSession = TypedRollSession<NumericRoll, NumericDice>;
pub type FudgeSession = TypedRollSession<FudgeRoll, FudgeDice>;

impl<T: Clone + Debug + Display, V: Debug + Clone + Display> TypedRollSession<T, V> {
    pub fn build(dice_requests: Vec<RollRequest<V>>) -> TypedRollSession<T, V>
    where
        Rolls<T, V>: Apply<T, V>,
        dice::DiceGenerator: dice::Roll<T, V>,
    {
        TypedRollSession::build_with_actions(
            dice_requests
                .into_iter()
                .map(|roll_request| RollAndActionsRequest::new(roll_request, vec![]))
                .collect::<Vec<RollAndActionsRequest<V>>>(),
        )
        // TODO for now, without action, there's no reason for it to fail. But who can know what the future holds?
        .expect("How did this happen to us?")
    }

    pub fn build_with_actions(
        requests: Vec<RollAndActionsRequest<V>>,
    ) -> Result<TypedRollSession<T, V>, Error>
    where
        Rolls<T, V>: Apply<T, V>,
        dice::DiceGenerator: dice::Roll<T, V>,
    {
        let dice = DiceGenerator::new();
        let rolls: Result<Vec<Rolls<T, V>>, Error> = requests
            .into_iter()
            .map(|dice_request| dice_request.roll(&dice))
            .collect();
        Ok(TypedRollSession {
            rolls: rolls?,
            dice,
        })
    }
}

pub trait Session: Debug + ToString {
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error>;

    fn write_results_to_file(&self, filepath: &str) -> std::io::Result<()> {
        let path = Path::new(filepath);

        let mut file = File::create(&path)?;
        file.write_all(self.to_string().as_bytes())
    }
}

impl Session for NumericSession {
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        match action {
            Action::Total => self.rolls = vec![self.rolls.total()],
            _ => {
                for rolls in self.rolls.iter_mut() {
                    *rolls = rolls.apply(&action, &self.dice)?;
                }
            }
        }
        Ok(())
    }
}

impl Session for FudgeSession {
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        for rolls in self.rolls.iter_mut() {
            *rolls = rolls.apply(&action, &self.dice)?;
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

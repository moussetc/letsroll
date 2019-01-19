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

#[derive(Debug)]
pub struct TypedRollSession<T: RollBounds, V: DiceBounds> {
    pub(crate) requests: Vec<RollRequest<V>>,
    pub rolls: Vec<Rolls<T, V>>,
    dice: DiceGenerator,
}

pub type NumericSession = TypedRollSession<NumericRoll, NumericDice>;
pub type FudgeSession = TypedRollSession<FudgeRoll, FudgeDice>;

impl<T: RollBounds, V: DiceBounds> TypedRollSession<T, V> {
    pub fn build(dice_requests: Vec<RollRequest<V>>) -> TypedRollSession<T, V>
    where
        Rolls<T, V>: Apply<T, V>,
        dice::DiceGenerator: dice::Roll<T, V>,
    {
        TypedRollSession::build_with_actions(dice_requests)
            // TODO for now, without action, there's no reason for it to fail. But who can know what the future holds?
            .expect("How did this happen to us?")
    }

    pub fn build_with_actions(
        requests: Vec<RollRequest<V>>,
    ) -> Result<TypedRollSession<T, V>, Error>
    where
        Rolls<T, V>: Apply<T, V>,
        dice::DiceGenerator: dice::Roll<T, V>,
    {
        let dice = DiceGenerator::new();
        let rolls: Result<Vec<Rolls<T, V>>, Error> = requests
            .iter()
            .map(|dice_request| dice_request.roll(&dice))
            .collect();
        Ok(TypedRollSession {
            requests: requests,
            rolls: rolls?,
            dice,
        })
    }
}

pub trait TransformableSession: Debug + ToString + Sized {
    fn add_transformation(&mut self, action: actions::Action) -> Result<(), Error>;

    fn add_actions(&mut self, actions: Vec<Action>) -> Result<(), Error> {
        for action in actions.into_iter() {
            self.add_transformation(action)?;
        }
        Ok(())
    }
}
impl TransformableSession for NumericSession {
    fn add_transformation(&mut self, action: actions::Action) -> Result<(), Error> {
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

impl TransformableSession for FudgeSession {
    fn add_transformation(&mut self, action: actions::Action) -> Result<(), Error> {
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

impl TransformableSession for MultiTypeSession {
    fn add_transformation(&mut self, action: actions::Action) -> Result<(), Error> {
        match &mut self.numeric_session {
            Some(ref mut session) => session.add_transformation(action.clone())?,
            None => (),
        };
        match &mut self.fudge_session {
            Some(ref mut session) => session.add_transformation(action.clone())?,
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
    //     assert_eq!(Ok(()), request.add_action(action));
    //     let output = request.dice_rolls;

    //     assert_eq!(output.len(), dice_requests_len);
    // }

}

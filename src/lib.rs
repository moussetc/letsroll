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
        if let Some(ref mut session) = &mut self.numeric_session {
            session.add_transformation(action.clone())?;
        }
        if let Some(ref mut session) = &mut self.fudge_session {
            session.add_transformation(action.clone())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use crate::actions::Action;
    // use crate::dice::{DiceGenerator, FudgeDice, FudgeRoll, FudgeRollRequest, NumericDice};
    // use crate::RollRequest;

    // TODO
}

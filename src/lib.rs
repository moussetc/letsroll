pub mod actions;
pub mod dice;
pub mod errors;

use crate::actions::{ActionKind, Transform};
use crate::dice::{DiceKind, Roll, RollResult};
use crate::errors::Error;
use std::fmt;
use std::str::FromStr;

pub struct DiceRequest {
    kind: DiceKind,
    number: u8,
}
impl DiceRequest {
    pub fn new(kind: DiceKind, number: u8) -> DiceRequest {
        DiceRequest { kind, number }
    }
}

impl FromStr for DiceRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
                    kind: DiceKind::NumberedDice(sides_fromstr),
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

// Dans un premier temps, on ne lance qu'un seul type de dés, merci bien.
pub struct RollRequest {
    dice: Vec<(DiceRequest, Box<dyn Roll>)>,
    steps: Vec<RollResult>,
}

impl RollRequest {
    pub fn new(dice_requests: Vec<DiceRequest>) -> RollRequest {
        let mut request = RollRequest {
            dice: dice_requests
                .into_iter()
                .map(|dice_request| {
                    let dice = RollRequest::select_dice(&dice_request.kind);
                    (dice_request, dice)
                })
                .collect(),
            steps: vec![],
        };
        {
            // Initial rolls
            for dice in request.dice.iter_mut() {
                let rolls = (0..dice.0.number).map(|_| dice.1.roll());
                request.steps.extend(rolls);
            }
        }
        request
    }

    pub fn add_step(&mut self, kind: actions::ActionKind) {
        let action_kind = RollRequest::select_action(&kind);
        self.steps = action_kind.transform(&self.steps);
    }

    pub fn results(&self) -> &Vec<RollResult> {
        &self.steps
    }

    fn select_dice(kind: &DiceKind) -> Box<dyn Roll> {
        match kind {
            DiceKind::Mock(mock_value) => Box::new(dice::Mock::new(*mock_value)) as Box<Roll>,
            DiceKind::NumberedDice(sides) => Box::new(dice::NumberedDice::new(*sides)) as Box<Roll>,
        }
    }

    fn select_action(kind: &ActionKind) -> Box<dyn Transform> {
        match kind {
            ActionKind::Identity => Box::new(actions::Identity {}) as Box<Transform>,
            ActionKind::FlipFlop => Box::new(actions::FlipFlop {}) as Box<Transform>,
            ActionKind::MultiplyBy(factor) => {
                Box::new(actions::MultiplyBy::new(*factor)) as Box<Transform>
            }
        }
    }
}

impl fmt::Display for RollRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Requested dice : {}\nResults: {}",
            self.dice
                .iter()
                .map(|request| request.0.to_string())
                .collect::<Vec<String>>()
                .join(" "),
            self.results()
                .iter()
                .map(|roll| roll.result.to_string())
                .collect::<Vec<String>>()
                .join(" ")
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
    // use crate::actions::ActionKind;
    use crate::dice::DiceKind;

    #[test]
    fn mock_request() {
        let dice_number = 5;
        let mock_value = 15;
        let request = crate::RollRequest::new(vec![crate::DiceRequest::new(
            DiceKind::Mock(mock_value),
            dice_number,
        )]);
        // request.add_step(ActionKind::FlipFlop);
        let output = request.results();

        assert_eq!(dice_number as usize, output.len());
    }

}

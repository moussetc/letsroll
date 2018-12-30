pub mod actions;
pub mod dice;
pub mod errors;

// use crate::actions::{ActionKind, Transform};
use crate::dice::{
    Dice, DiceKind, FudgeDice, NumberedDice, NumericDice, NumericRoll, Roll, RollEnum, TextDice,
    TextRoll,
};
use crate::errors::Error;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash)]
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
        if s.ends_with("F") {
            let number_from_str = s[0..s.len() - 1].parse::<u8>()?;
            return Ok(DiceRequest {
                kind: DiceKind::TextDice(TextDice::FudgeDice(FudgeDice::new())),
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
                    kind: DiceKind::NumericDice(NumericDice::NumberedDice(NumberedDice::new(
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
    rolls: HashMap<DiceRequest, Vec<RollEnum>>,
}

impl RollRequest {
    pub fn new(dice_requests: Vec<DiceRequest>) -> RollRequest {
        let mut request = RollRequest {
            rolls: HashMap::new(),
        };
        // Initial rolls

        let requests = dice_requests;
        {
            for mut dice in requests.into_iter() {
                let mut rolls: Vec<RollEnum> = vec![];
                for _ in 0..dice.number {
                    let roll = match dice.kind {
                        DiceKind::NumericDice(ref mut num_dice) => {
                            RollEnum::NumericRoll(num_dice.roll())
                        }
                        DiceKind::TextDice(ref mut dice) => RollEnum::TextRoll(dice.roll()),
                    };
                    rolls.push(roll);
                }
                request.rolls.insert(dice, rolls);
            }
        }

        // let rolls = (0..dice.number).map(|_| match dice.kind {
        //     DiceKind::NumericDice(ref mut num_dice) => {
        //         RollEnum::NumericRoll(num_dice.roll())
        //     }
        //     DiceKind::TextDice(ref mut dice) => RollEnum::TextRoll(dice.roll()),
        // });

        // request.steps.extend(rolls);
        request
    }

    // pub fn add_step(&mut self, kind: actions::ActionKind) {
    //     let action_kind = RollRequest::select_action(&kind);
    //     self.steps = action_kind.transform(&self.steps);
    // }

    pub fn results(&self) -> &HashMap<DiceRequest, Vec<RollEnum>> {
        &self.rolls
    }

    // fn select_action(kind: &ActionKind) -> Box<dyn Transform> {
    //     match kind {
    //         ActionKind::Identity => Box::new(actions::IdentityOld {}) as Box<Transform>,
    //         ActionKind::FlipFlop => Box::new(actions::FlipFlop {}) as Box<Transform>,
    //         ActionKind::MultiplyBy(factor) => {
    //             Box::new(actions::MultiplyBy::new(*factor)) as Box<Transform>
    //         }
    //     }
    // }
}

impl fmt::Display for RollRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.results()
                .iter()
                .map(|keyval| format!(
                    "{} : {}|",
                    keyval.0,
                    keyval
                        .1
                        .iter()
                        .map(|roll| roll.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                ))
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
    // use crate::actions::ActionKind;
    use crate::dice::{DiceKind, Mock, NumericDice};

    #[test]
    fn mock_request() {
        let dice_number = 5;
        let mock_val = 15;

        let dice_requests = vec![crate::DiceRequest::new(
            DiceKind::NumericDice(NumericDice::Mock(Mock::new(mock_val))),
            dice_number,
        )];
        let dice_requests_len = dice_requests.len();

        let request = crate::RollRequest::new(dice_requests);
        // request.add_step(ActionKind::FlipFlop);
        let output = request.results();

        assert_eq!(output.len(), dice_requests_len);
        for keyval in output.iter() {
            match &keyval.0.kind {
                DiceKind::NumericDice(num_dice) => match num_dice {
                    NumericDice::Mock(Mock { mock_value, .. }) => assert_eq!(mock_val, *mock_value),
                    _ => assert!(false, "Wrong dice kind"),
                },
                _ => assert!(false, "Wrong dice kind"),
            }
        }
    }

}

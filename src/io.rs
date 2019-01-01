use crate::dice::*;
use crate::errors::{Error, ErrorKind};
use crate::{DiceRequest, RollRequest};
use std::fmt;
use std::str::FromStr;

impl FromStr for DiceRequest {
    type Err = Error;
    // Parse a string to find a dice definition (number of dice + dice type + optional dice parameters)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("F") {
            let number_from_str = match s.len() {
                1 => 1,
                _ => s[0..s.len() - 1].parse::<DiceNumber>()?,
            };
            return Ok(DiceRequest {
                kind: DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())),
                number: number_from_str,
            });
        }

        if s.starts_with("+") {
            let number_from_str = s[..s.len()].parse::<NumericRoll>()?;
            return Ok(DiceRequest {
                kind: DiceKind::NumericKind(NumericDice::Const(Const::new(number_from_str))),
                number: 1,
            });
        }

        if s.contains("D") {
            let parts: Vec<&str> = s.trim().split('D').collect();

            match parts.len() {
                1 | 2 => {
                    // Format: D6 or 3D6
                    let mut number_fromstr = parts[0].parse::<u8>()?;
                    let sides_fromstr = parts[parts.len() - 1].parse::<u16>()?;
                    // the number of dice to roll is optional and defaults to 1
                    if parts.len() == 1 {
                        number_fromstr = 1;
                    }
                    return Ok(DiceRequest {
                        number: number_fromstr,
                        kind: DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(
                            sides_fromstr,
                        ))),
                    });
                }
                _ => Err(Error::new(ErrorKind::ParseDice(format!(
                    "Expected something like 'D20' or '3D6' but got \"{}\" ",
                    s
                )))),
            }
        } else {
            Err(Error::new(ErrorKind::ParseDice(format!(
                "\"{}\" does not parse to any known dice",
                s
            ))))
        }
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
                Rolls::TextRolls(rolls) => rolls
                    .iter()
                    .map(|roll| roll.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
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
                DiceKind::NumericKind(num_dice) => match num_dice {
                    NumericDice::Const(dice) => format!("x{}", dice.const_value),
                    NumericDice::NumberedDice(dice) => format!("D{}", dice.get_param()),
                },
                DiceKind::TextKind(text_dice) => match text_dice {
                    TextDice::FudgeDice(_) => String::from("F"),
                    TextDice::Const(d) => d.const_value.to_string(),
                },
            }
        )
    }
}

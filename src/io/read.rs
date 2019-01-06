use crate::dice::*;
use crate::errors::{Error, ErrorKind};
use crate::{DiceRequest, RollRequest};
use std::str::FromStr;

impl FromStr for DiceRequest {
    type Err = Error;
    // Parse a string to find a dice definition (number of dice + dice type + optional dice parameters)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim().to_uppercase();
        if input.ends_with("F") {
            let number_from_str = match input.len() {
                1 => 1,
                _ => input[0..input.len() - 1].parse::<DiceNumber>()?,
            };
            return Ok(DiceRequest {
                kind: DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())),
                number: number_from_str,
            });
        }

        if input.starts_with("+") {
            let number_from_str = input[..input.len()].parse::<NumericRoll>()?;
            return Ok(DiceRequest {
                kind: DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(
                    number_from_str,
                ))),
                number: 1,
            });
        }

        if input.contains("D") {
            let parts: Vec<&str> = input.split('D').collect();

            match parts.len() {
                1 | 2 => {
                    // Format: D6 or 3D6
                    let mut number_fromstr = match parts[0].len() {
                        0 => 1,
                        _ => parts[0].parse::<u8>()?,
                    };
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
    use crate::dice::*;
    use crate::DiceRequest;
    use std::str::FromStr;

    #[test]
    fn read_numbered_dice() {
        assert_eq!(
            DiceRequest::from_str(&String::from("5d6")).unwrap(),
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(6))),
                5
            )
        );

        assert_eq!(
            DiceRequest::from_str(&String::from("8D3")).unwrap(),
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(3))),
                8
            )
        );

        assert_eq!(
            DiceRequest::from_str(&String::from("D20")).unwrap(),
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(20))),
                1
            )
        );
    }

    #[test]
    fn read_fudge_dice() {
        assert_eq!(
            DiceRequest::from_str(&String::from("F")).unwrap(),
            DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 1)
        );

        assert_eq!(
            DiceRequest::from_str(&String::from("8F")).unwrap(),
            DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 8)
        );
    }

    #[test]
    fn read_ko() {
        DiceRequest::from_str(&String::from("5")).unwrap_err();
        DiceRequest::from_str(&String::from("Da")).unwrap_err();
        DiceRequest::from_str(&String::from("D8D")).unwrap_err();
        DiceRequest::from_str(&String::from("F8")).unwrap_err();
        DiceRequest::from_str(&String::from("+")).unwrap_err();
        DiceRequest::from_str(&String::from("8+")).unwrap_err();
        DiceRequest::from_str(&String::from("+8+")).unwrap_err();
        DiceRequest::from_str(&String::from("2+8")).unwrap_err();
    }

}

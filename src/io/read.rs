use crate::dice::*;
use crate::errors::Error;
use crate::{DiceRequest, RollRequest};
use std::str::FromStr;

use pest::Parser;

#[derive(Parser)]
#[grammar = "roll_request.pest"]
pub struct RequestParser;

impl FromStr for RollRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RollRequest::new(parse_request(s)?))
    }
}

fn parse_request(s: &str) -> Result<Vec<DiceRequest>, Error> {
    match RequestParser::parse(Rule::roll_request, s) {
        Err(err) => Err(Error::from(err)),
        Ok(mut parsed_roll_request) => {
            let mut request_dice: Vec<DiceRequest> = vec![];
            for dice_or_action in parsed_roll_request.next().unwrap().into_inner() {
                match dice_or_action.as_rule() {
                    Rule::dice => {
                        for dice in dice_or_action.into_inner() {
                            match dice.as_rule() {
                                Rule::fudge_dice => {
                                    let mut inner_rules = dice.into_inner();
                                    let mut dice_number: DiceNumber = 1;
                                    match inner_rules.next() {
                                        Some(rule) => match rule.as_rule() {
                                            Rule::dice_number => {
                                                dice_number =
                                                    rule.as_str().parse::<DiceNumber>().unwrap();
                                            }
                                            _ => unreachable!(),
                                        },
                                        None => (),
                                    }
                                    request_dice.push(DiceRequest::new(
                                        DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())),
                                        dice_number,
                                    ));
                                }
                                Rule::num_const_dice => {
                                    let const_value: NumericRoll;
                                    let rule = dice.into_inner().next().unwrap();
                                    match rule.as_rule() {
                                        Rule::dice_sides => {
                                            const_value =
                                                rule.as_str().parse::<NumericRoll>().unwrap();
                                        }
                                        _ => unreachable!(),
                                    }
                                    request_dice.push(DiceRequest::new(
                                        DiceKind::NumericKind(NumericDice::ConstDice(
                                            ConstDice::new(const_value),
                                        )),
                                        1,
                                    ));
                                }
                                Rule::numbered_dice => {
                                    let mut dice_number: DiceNumber = 1;
                                    let mut dice_sides: NumericRoll = 1;
                                    for rule in dice.into_inner() {
                                        match rule.as_rule() {
                                            Rule::dice_number => {
                                                dice_number =
                                                    rule.as_str().parse::<DiceNumber>().unwrap();
                                            }
                                            Rule::dice_sides => {
                                                dice_sides =
                                                    rule.as_str().parse::<NumericRoll>().unwrap();
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                    request_dice.push(DiceRequest::new(
                                        DiceKind::NumericKind(NumericDice::NumberedDice(
                                            NumberedDice::new(dice_sides),
                                        )),
                                        dice_number,
                                    ));
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    Rule::EOI => (),
                    _ => unreachable!(),
                }
            }
            Ok(request_dice)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::*;
    use crate::io::read::parse_request;
    use crate::DiceRequest;

    #[test]
    fn read_numbered_dice() {
        assert_eq!(
            parse_request(&String::from("5d6")).unwrap()[0],
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(6))),
                5
            )
        );

        assert_eq!(
            parse_request(&String::from("8D3")).unwrap()[0],
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(3))),
                8
            )
        );

        assert_eq!(
            parse_request(&String::from("D20")).unwrap()[0],
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(20))),
                1
            )
        );
    }

    #[test]
    fn read_fudge_dice() {
        assert_eq!(
            parse_request(&String::from("F")).unwrap()[0],
            DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 1)
        );

        assert_eq!(
            parse_request(&String::from("8F")).unwrap()[0],
            DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 8)
        );
    }

    #[test]
    fn read_const_dice() {
        assert_eq!(
            parse_request(&String::from("+5")).unwrap()[0],
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(5))),
                1
            )
        );

        assert_eq!(
            parse_request(&String::from("+100")).unwrap()[0],
            DiceRequest::new(
                DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(100))),
                1
            )
        );
    }

    #[test]
    fn read_ko() {
        parse_request(&String::from("5")).unwrap_err();
        parse_request(&String::from("Da")).unwrap_err();
        parse_request(&String::from("D8D")).unwrap_err();
        parse_request(&String::from("F8")).unwrap_err();
        parse_request(&String::from("+")).unwrap_err();
        parse_request(&String::from("8+")).unwrap_err();
        parse_request(&String::from("+8+")).unwrap_err();
        parse_request(&String::from("2+8")).unwrap_err();
        parse_request(&String::from("5D 20")).unwrap_err();
    }

}

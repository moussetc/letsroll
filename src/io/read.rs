use crate::actions::Action;
use crate::dice::*;
use crate::errors::Error;
use crate::{FullRollSession, RollSession, Session};
use std::str::FromStr;

use pest::Parser;

#[derive(Parser)]
#[grammar = "roll_request.pest"]
pub struct RequestParser;

pub fn parse_request(s: &str) -> Result<FullRollSession, Error> {
    match RequestParser::parse(Rule::roll_request, s) {
        Err(err) => Err(Error::from(err)),
        Ok(mut parsed_roll_request) => {
            let mut num_request_dice: Vec<DiceRequest<NumericDice>> = vec![];
            let mut fudge_request_dice: Vec<DiceRequest<FudgeDice>> = vec![];
            let mut actions: Vec<Action> = vec![];
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
                                    fudge_request_dice
                                        .push(DiceRequest::new(dice_number, FudgeDice::FudgeDice));
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
                                    num_request_dice.push(DiceRequest::new(
                                        1,
                                        NumericDice::ConstDice(const_value),
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
                                    num_request_dice.push(DiceRequest::new(
                                        dice_number,
                                        NumericDice::NumberedDice(dice_sides),
                                    ));
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    Rule::action => {
                        for action in dice_or_action.into_inner() {
                            match action.as_rule() {
                                // TODO "Sum" after the dice is "total sum" which has to be implemented
                                Rule::action_sum => actions.push(Action::Sum),
                                Rule::action_flip => actions.push(Action::FlipFlop),
                                Rule::action_total => actions.push(Action::Total),
                                // TODO : add other actions
                                _ => unreachable!(),
                            }
                        }
                    }
                    Rule::EOI => (),
                    _ => unreachable!(),
                }
            }

            let mut sessions: Vec<Box<dyn Session>> = vec![];
            if num_request_dice.len() > 0 {
                let mut session = RollSession::<NumericRoll, NumericDice>::new(num_request_dice);
                for action in actions.iter() {
                    session.add_step(*action)?;
                }
                sessions.push(Box::new(session));
            }
            if fudge_request_dice.len() > 0 {
                let mut session = RollSession::<FudgeRoll, FudgeDice>::new(fudge_request_dice);
                for action in actions.iter() {
                    session.add_step(*action)?;
                }
                sessions.push(Box::new(session));
            }

            Ok(FullRollSession::new(sessions))
        }
    }
}

#[cfg(test)]
mod tests {
    // use crate::dice::*;
    // use crate::io::read::parse_request;
    // use crate::DiceRequest;

    // #[test]
    // fn read_numbered_dice() {
    //     assert_eq!(
    //         parse_request(&String::from("5d6")).unwrap().0[0],
    //         DiceRequest::new(
    //             DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(6))),
    //             5
    //         )
    //     );

    //     assert_eq!(
    //         parse_request(&String::from("8D3")).unwrap().0[0],
    //         DiceRequest::new(
    //             DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(3))),
    //             8
    //         )
    //     );

    //     assert_eq!(
    //         parse_request(&String::from("D20")).unwrap().0[0],
    //         DiceRequest::new(
    //             DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(20))),
    //             1
    //         )
    //     );
    // }

    // #[test]
    // fn read_fudge_dice() {
    //     assert_eq!(
    //         parse_request(&String::from("F")).unwrap().0[0],
    //         DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 1)
    //     );

    //     assert_eq!(
    //         parse_request(&String::from("8F")).unwrap().0[0],
    //         DiceRequest::new(DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new())), 8)
    //     );
    // }

    // #[test]
    // fn read_const_dice() {
    //     assert_eq!(
    //         parse_request(&String::from("+5")).unwrap().0[0],
    //         DiceRequest::new(
    //             DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(5))),
    //             1
    //         )
    //     );

    //     assert_eq!(
    //         parse_request(&String::from("+100")).unwrap().0[0],
    //         DiceRequest::new(
    //             DiceKind::NumericKind(NumericDice::ConstDice(ConstDice::new(100))),
    //             1
    //         )
    //     );
    // }

    // // TODO add test for global actions + dice actions

    // #[test]
    // fn read_ko() {
    //     parse_request(&String::from("5")).unwrap_err();
    //     parse_request(&String::from("Da")).unwrap_err();
    //     parse_request(&String::from("D8D")).unwrap_err();
    //     parse_request(&String::from("F8")).unwrap_err();
    //     parse_request(&String::from("+")).unwrap_err();
    //     parse_request(&String::from("8+")).unwrap_err();
    //     parse_request(&String::from("+8+")).unwrap_err();
    //     parse_request(&String::from("2+8")).unwrap_err();
    //     parse_request(&String::from("5D 20")).unwrap_err();
    // }

}

use crate::actions::Action;
use crate::dice::*;
use crate::errors::{Error, ErrorKind};
use crate::MultiTypeSession;
use crate::{FudgeSession, NumericSession, Session};
use std::str::FromStr;

use pest::Parser;

#[derive(Parser)]
#[grammar = "roll_request.pest"]
pub struct RequestParser;

impl FromStr for FudgeRoll {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            x if x == "+" => Ok(FudgeRoll::Plus),
            x if x == "-" => Ok(FudgeRoll::Minus),
            x if x == "0" => Ok(FudgeRoll::Blank),
            _ => Err(Error::new(ErrorKind::Parse(format!(
                "Can't read '{}' as a fudge roll value",
                s
            )))),
        }
    }
}

impl FromStr for NumericSession {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_request(s)?
            .numeric_session
            .ok_or(Error::new(ErrorKind::Parse(String::from(
                "Could not parse numeric roll request",
            ))))
    }
}

impl FromStr for FudgeSession {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_request(s)?
            .fudge_session
            .ok_or(Error::new(ErrorKind::Parse(String::from(
                "Could not parse fudge roll request",
            ))))
    }
}

pub fn parse_request(s: &str) -> Result<MultiTypeSession, Error> {
    match RequestParser::parse(Rule::roll_request, s) {
        Err(err) => Err(Error::from(err)),
        Ok(mut parsed_roll_request) => {
            let mut num_request_dice: Vec<NumericRollRequest> = vec![];
            let mut fudge_request_dice: Vec<FudgeRollRequest> = vec![];
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
                                        .push(RollRequest::new(dice_number, FudgeDice::FudgeDice));
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
                                    num_request_dice.push(RollRequest::new(
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
                                    num_request_dice.push(RollRequest::new(
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
                                Rule::action_sum => actions.push(Action::Total),
                                Rule::action_flip => actions.push(Action::FlipFlop),
                                Rule::action_total => actions.push(Action::Total),

                                Rule::action_mult => {
                                    let factor: NumericRoll;
                                    let rule = action.into_inner().next().unwrap();
                                    match rule.as_rule() {
                                        Rule::factor => {
                                            factor = rule.as_str().parse::<NumericRoll>().unwrap();
                                        }
                                        _ => unreachable!(),
                                    }
                                    actions.push(Action::MultiplyBy(factor));
                                }
                                Rule::action_reroll => {
                                    let rule = action.into_inner().next().unwrap();
                                    match rule.as_rule() {
                                        Rule::num_roll_value => {
                                            let reroll_value =
                                                rule.as_str().parse::<NumericRoll>().unwrap();
                                            actions.push(Action::RerollNumeric(reroll_value));
                                        }
                                        Rule::fudge_roll_value => {
                                            let reroll_value =
                                                rule.as_str().parse::<FudgeRoll>().unwrap();
                                            actions.push(Action::RerollFudge(reroll_value));
                                        }
                                        _ => unreachable!(),
                                    };
                                }
                                Rule::action_explode => {
                                    let rule = action.into_inner().next().unwrap();
                                    match rule.as_rule() {
                                        Rule::num_roll_value => {
                                            let explode_value =
                                                rule.as_str().parse::<NumericRoll>().unwrap();
                                            actions.push(Action::Explode(explode_value));
                                        }
                                        Rule::fudge_roll_value => {
                                            let explode_value =
                                                rule.as_str().parse::<FudgeRoll>().unwrap();
                                            actions.push(Action::ExplodeFudge(explode_value));
                                        }
                                        _ => unreachable!(),
                                    };
                                }
                                // TODO : add other actions
                                _ => unreachable!(),
                            }
                        }
                    }
                    Rule::EOI => (),
                    _ => unreachable!(),
                }
            }

            let mut res = MultiTypeSession {
                numeric_session: None,
                fudge_session: None,
            };

            if num_request_dice.len() > 0 {
                let mut session = NumericSession::new(num_request_dice);
                for action in actions.iter() {
                    session.add_step(*action)?;
                }
                res.numeric_session = Some(session);
            }
            if fudge_request_dice.len() > 0 {
                let mut session = FudgeSession::new(fudge_request_dice);
                for action in actions.iter() {
                    session.add_step(*action)?;
                }
                res.fudge_session = Some(session);
            }

            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::*;
    use crate::io::read::parse_request;
    use crate::FudgeSession;
    use crate::NumericSession;
    use std::str::FromStr;

    #[test]
    fn read_numbered_dice() {
        let dice_request = &NumericSession::from_str(&String::from("5d6"))
            .unwrap()
            .rolls[0]
            .dice_request;
        assert_eq!(dice_request.dice, NumericDice::NumberedDice(6));
        assert_eq!(dice_request.number, 5);

        let dice_request = &NumericSession::from_str(&String::from("8D3"))
            .unwrap()
            .rolls[0]
            .dice_request;
        assert_eq!(dice_request.dice, NumericDice::NumberedDice(3));
        assert_eq!(dice_request.number, 8);

        let dice_request = &NumericSession::from_str(&String::from("D20"))
            .unwrap()
            .rolls[0]
            .dice_request;
        assert_eq!(dice_request.dice, NumericDice::NumberedDice(20));
        assert_eq!(dice_request.number, 1);
    }

    #[test]
    fn read_fudge_dice() {
        let dice_request =
            &FudgeSession::from_str(&String::from("F")).unwrap().rolls[0].dice_request;
        assert_eq!(dice_request.dice, FudgeDice::FudgeDice);
        assert_eq!(dice_request.number, 1);

        let dice_request =
            &FudgeSession::from_str(&String::from("10F")).unwrap().rolls[0].dice_request;
        assert_eq!(dice_request.dice, FudgeDice::FudgeDice);
        assert_eq!(dice_request.number, 10);
    }

    #[test]
    fn read_const_dice() {
        let dice_request =
            &NumericSession::from_str(&String::from("+5")).unwrap().rolls[0].dice_request;
        assert_eq!(dice_request.dice, NumericDice::ConstDice(5));
        assert_eq!(dice_request.number, 1);

        let dice_request = &NumericSession::from_str(&String::from("+142"))
            .unwrap()
            .rolls[0]
            .dice_request;
        assert_eq!(dice_request.dice, NumericDice::ConstDice(142));
        assert_eq!(dice_request.number, 1);
    }

    // // TODO add test for global actions + dice actions + KO tests for incompatibility

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

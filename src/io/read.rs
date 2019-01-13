use crate::actions::Action;
use crate::actions::Aggregation;
use crate::dice::*;
use crate::errors::{Error, ErrorKind};
use crate::MultiTypeSession;
use crate::{AggregatableSession, FudgeSession, NumericSession, Session};
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
        parse_request(s, false)?
            .numeric_session
            .ok_or(Error::new(ErrorKind::Parse(String::from(
                "Could not parse numeric roll request",
            ))))
    }
}

impl FromStr for FudgeSession {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_request(s, false)?
            .fudge_session
            .ok_or(Error::new(ErrorKind::Parse(String::from(
                "Could not parse fudge roll request",
            ))))
    }
}

/// Try to parse a roll request from an input String.
/// Roll sessions are created for each type of dice present (numeric and fudge dice don't mix).AggregatableSession
///
/// # Arguments
/// * `s` Input string
/// * `default_total` If set to `true`, in the absence of a parsed aggregation, the `ToTal` action will be applied to numeric rolls.
/// This is allows users not to have to specify the Sum action each time they do a classic roll requiring the total.
pub fn parse_request(s: &str, default_total: bool) -> Result<MultiTypeSession, Error> {
    match RequestParser::parse(Rule::roll_request, s) {
        Err(err) => Err(Error::from(err)),
        Ok(mut parsed_roll_request) => {
            let mut num_request_dice: Vec<NumericRollRequest> = vec![];
            let mut fudge_request_dice: Vec<FudgeRollRequest> = vec![];
            let mut aggregation: Option<Aggregation> = None;
            let mut actions: Vec<Action> = vec![];
            for dice_or_action in parsed_roll_request.next().unwrap().into_inner() {
                match dice_or_action.as_rule() {
                    Rule::dice => {
                        for dice in dice_or_action.into_inner() {
                            match dice.as_rule() {
                                Rule::fudge_dice => {
                                    fudge_request_dice.push(parse_fudge_dice(dice)?);
                                }
                                Rule::num_const_dice => {
                                    num_request_dice.push(parse_const_numeric_dice(dice)?)
                                }
                                Rule::numbered_dice => {
                                    num_request_dice.push(parse_numbered_dice(dice)?);
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
                                    actions.push(parse_multiply_action(action)?);
                                }
                                Rule::action_reroll => {
                                    actions.push(parse_reroll_action(action)?);
                                }
                                Rule::action_explode => {
                                    actions.push(parse_explode_action(action)?);
                                }
                                // TODO : add other actions
                                _ => unreachable!(),
                            }
                        }
                    }
                    Rule::aggregation => {
                        for aggreg_action in dice_or_action.into_inner() {
                            match aggreg_action.as_rule() {
                                Rule::aggregation_count => {
                                    aggregation = Some(Aggregation::CountValues)
                                }
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
                    session.add_step(action.clone())?;
                }
                if aggregation.is_some() {
                    session = session.aggregate(&aggregation.unwrap());
                } else if default_total && aggregation.is_none() && actions.len() == 0 {
                    session.add_step(Action::Total)?;
                }
                res.numeric_session = Some(session);
            }
            if fudge_request_dice.len() > 0 {
                let mut session = FudgeSession::new(fudge_request_dice);
                for action in actions.iter() {
                    session.add_step(action.clone())?;
                }
                if aggregation.is_some() {
                    let mut num_session = session.aggregate(&aggregation.unwrap());
                    let res_mut = &mut res;
                    if res_mut.numeric_session.is_some() {
                        // TODO somehow merge the two numeric sessions
                        // unimplemented!();
                        res_mut
                            .numeric_session
                            .as_mut()
                            .unwrap()
                            .rolls
                            .append(&mut num_session.rolls);
                    } else {
                        res.numeric_session = Some(num_session);
                    }
                } else {
                    res.fudge_session = Some(session);
                }
            }

            Ok(res)
        }
    }
}

fn parse_numbered_dice(dice: pest::iterators::Pair<'_, Rule>) -> Result<NumericRollRequest, Error> {
    let mut dice_number: DiceNumber = 1;
    let mut dice_sides: NumericRoll = 1;
    for rule in dice.into_inner() {
        match rule.as_rule() {
            Rule::dice_number => {
                dice_number = rule.as_str().parse::<DiceNumber>().unwrap();
            }
            Rule::dice_sides => {
                dice_sides = rule.as_str().parse::<NumericRoll>().unwrap();
            }
            _ => unreachable!(),
        }
    }
    Ok(RollRequest::new(
        dice_number,
        NumericDice::NumberedDice(dice_sides),
    ))
}

fn parse_const_numeric_dice(
    dice: pest::iterators::Pair<'_, Rule>,
) -> Result<NumericRollRequest, Error> {
    let const_value: NumericRoll;
    let rule = dice.into_inner().next().unwrap();
    match rule.as_rule() {
        Rule::dice_sides => {
            const_value = rule.as_str().parse::<NumericRoll>().unwrap();
        }
        _ => unreachable!(),
    }
    Ok(RollRequest::new(1, NumericDice::ConstDice(const_value)))
}

fn parse_fudge_dice(dice: pest::iterators::Pair<'_, Rule>) -> Result<FudgeRollRequest, Error> {
    let mut dice_number: DiceNumber = 1;
    for rule in dice.into_inner() {
        match rule.as_rule() {
            Rule::dice_number => {
                dice_number = rule.as_str().parse::<DiceNumber>().unwrap();
            }
            _ => unreachable!(),
        }
    }
    Ok(RollRequest::new(dice_number, FudgeDice::FudgeDice))
}

fn parse_reroll_action(action: pest::iterators::Pair<'_, Rule>) -> Result<Action, Error> {
    let mut num_values: Vec<NumericRoll> = vec![];
    let mut fudge_values: Vec<FudgeRoll> = vec![];
    for rule in action.into_inner() {
        match rule.as_rule() {
            Rule::num_roll_value => {
                num_values.push(rule.as_str().parse::<NumericRoll>()?);
            }
            Rule::fudge_roll_value => {
                fudge_values.push(rule.as_str().parse::<FudgeRoll>()?);
            }
            _ => unreachable!(),
        }
    }
    // The grammar syntax enforce that there is at least one value
    // and that only values of the same type are present.
    match num_values.len() {
        x if x > 0 => Ok(Action::RerollNumeric(num_values)),
        _ => Ok(Action::RerollFudge(fudge_values)),
    }
}

fn parse_explode_action(action: pest::iterators::Pair<'_, Rule>) -> Result<Action, Error> {
    let mut num_values: Vec<NumericRoll> = vec![];
    let mut fudge_values: Vec<FudgeRoll> = vec![];
    for rule in action.into_inner() {
        match rule.as_rule() {
            Rule::num_roll_value => {
                num_values.push(rule.as_str().parse::<NumericRoll>()?);
            }
            Rule::fudge_roll_value => {
                fudge_values.push(rule.as_str().parse::<FudgeRoll>()?);
            }
            _ => unreachable!(),
        }
    }
    // The grammar syntax enforce that there is at least one value
    // and that only values of the same type are present.
    match num_values.len() {
        x if x > 0 => Ok(Action::Explode(num_values)),
        _ => Ok(Action::ExplodeFudge(fudge_values)),
    }
}

fn parse_multiply_action(action: pest::iterators::Pair<'_, Rule>) -> Result<Action, Error> {
    let rule = action.into_inner().next().unwrap();
    match rule.as_rule() {
        Rule::factor => Ok(Action::MultiplyBy(rule.as_str().parse::<NumericRoll>()?)),
        _ => unreachable!(),
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
        parse_request(&String::from("5"), false).unwrap_err();
        parse_request(&String::from("Da"), false).unwrap_err();
        parse_request(&String::from("D8D"), false).unwrap_err();
        parse_request(&String::from("F8"), false).unwrap_err();
        parse_request(&String::from("+"), false).unwrap_err();
        parse_request(&String::from("8+"), false).unwrap_err();
        parse_request(&String::from("+8+"), false).unwrap_err();
        parse_request(&String::from("2+8"), false).unwrap_err();
        parse_request(&String::from("5D 20"), false).unwrap_err();
    }

}

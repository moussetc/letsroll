use crate::actions::Action;
use crate::actions::Aggregation;
use crate::dice::*;
use crate::errors::{Error, ErrorKind};
use crate::MultiTypeSession;
use crate::{AggregatableSession, FudgeSession, NumericSession, TransformableSession};
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
                            let parsed_dice = parse_dice(dice)?;
                            if let Some(dice) = parsed_dice.0 {
                                num_request_dice.push(dice);
                            }
                            if let Some(dice) = parsed_dice.1 {
                                fudge_request_dice.push(dice);
                            }
                        }
                    }
                    Rule::dice_and_action => {
                        let mut dice_id: Option<String> = None;
                        let mut dice: Option<(
                            Option<NumericRollRequest>,
                            Option<FudgeRollRequest>,
                        )> = None;
                        let mut dice_actions: Vec<Action> = vec![];
                        for dice_or_dice_action in dice_or_action.into_inner() {
                            match dice_or_dice_action.as_rule() {
                                Rule::DICE_ID => {
                                    dice_id = Some(dice_or_dice_action.as_str().to_string());
                                }
                                Rule::dice => {
                                    dice = Some(parse_dice(
                                        dice_or_dice_action.into_inner().next().unwrap(),
                                    )?);
                                }
                                Rule::action => {
                                    parse_action(
                                        dice_or_dice_action.into_inner().next().unwrap(),
                                        &mut dice_actions,
                                    )?;
                                }
                                _ => unreachable!(),
                            }
                        }
                        if let Some(num_dice) = &dice.as_ref().unwrap().0 {
                            num_request_dice
                                .push(num_dice.clone().add_actions(dice_actions).add_id(dice_id));
                            continue;
                        }
                        if let Some(fudge_dice) = &dice.as_ref().unwrap().1 {
                            fudge_request_dice
                                .push(fudge_dice.clone().add_actions(dice_actions).add_id(dice_id));
                        }
                    }
                    Rule::action => {
                        for action in dice_or_action.into_inner() {
                            parse_action(action, &mut actions)?;
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
                let mut session = NumericSession::build_with_actions(num_request_dice)?;
                session.add_actions(actions.clone())?;
                if aggregation.is_some() {
                    session = session.aggregate(&aggregation.unwrap());
                } else if default_total && aggregation.is_none() && actions.len() == 0 {
                    session.add_transformation(Action::Total)?;
                }
                res.numeric_session = Some(session);
            }
            if fudge_request_dice.len() > 0 {
                let mut session = FudgeSession::build_with_actions(fudge_request_dice)?;
                session.add_actions(actions)?;
                if aggregation.is_some() {
                    let mut num_session = session.aggregate(&aggregation.unwrap());
                    let res_mut = &mut res;
                    if res_mut.numeric_session.is_some() {
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

fn parse_dice(
    dice: pest::iterators::Pair<'_, Rule>,
) -> Result<(Option<NumericRollRequest>, Option<FudgeRollRequest>), Error> {
    match dice.as_rule() {
        Rule::fudge_dice => Ok((None, Some(parse_fudge_dice(dice)?))),
        Rule::num_const_dice => Ok((Some(parse_const_numeric_dice(dice)?), None)),
        Rule::numbered_dice => Ok((Some(parse_numbered_dice(dice)?), None)),
        _ => unreachable!(),
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

fn parse_action(
    action: pest::iterators::Pair<'_, Rule>,
    actions: &mut Vec<Action>,
) -> Result<(), Error> {
    match action.as_rule() {
        Rule::action_sum => actions.push(Action::Total),
        Rule::action_flip => actions.push(Action::FlipFlop),
        Rule::action_total => actions.push(Action::Total),
        Rule::action_concat => actions.push(Action::Concat),
        Rule::action_mult => {
            actions.push(Action::MultiplyBy(parse_positive_int(action)?));
        }
        Rule::action_reroll => {
            actions.push(parse_reroll_action(action)?);
        }
        Rule::action_explode => {
            actions.push(parse_explode_action(action)?);
        }
        Rule::action_keep_best => {
            actions.push(Action::KeepBest(parse_positive_int(action)? as DiceNumber));
        }
        Rule::action_keep_worst => {
            actions.push(Action::KeepWorst(parse_positive_int(action)? as DiceNumber));
        }
        Rule::action_reroll_best => {
            actions.push(Action::RerollBest(parse_positive_int(action)? as DiceNumber));
        }
        Rule::action_reroll_worst => {
            actions.push(Action::RerollWorst(
                parse_positive_int(action)? as DiceNumber
            ));
        }
        _ => unreachable!(),
    };
    Ok(())
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

fn parse_positive_int(action: pest::iterators::Pair<'_, Rule>) -> Result<u32, Error> {
    let rule = action.into_inner().next().unwrap();
    match rule.as_rule() {
        Rule::POSITIVE_INT => Ok(rule.as_str().parse()?),
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
        let requests = &NumericSession::from_str(&String::from("5d6"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(5, NumericDice::NumberedDice(6))]
        );

        let requests = &NumericSession::from_str(&String::from("8D3"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(8, NumericDice::NumberedDice(3))]
        );

        let requests = &NumericSession::from_str(&String::from("D20"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(1, NumericDice::NumberedDice(20))]
        );
    }

    #[test]
    fn read_fudge_dice() {
        let requests = &FudgeSession::from_str(&String::from("F")).unwrap().requests;
        assert_eq!(*requests, vec![RollRequest::new(1, FudgeDice::FudgeDice)]);

        let requests = &FudgeSession::from_str(&String::from("10F"))
            .unwrap()
            .requests;
        assert_eq!(*requests, vec![RollRequest::new(10, FudgeDice::FudgeDice)]);
    }

    #[test]
    fn read_const_dice() {
        let requests = &NumericSession::from_str(&String::from("+5"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(1, NumericDice::ConstDice(5))]
        );

        let requests = &NumericSession::from_str(&String::from("+142"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(1, NumericDice::ConstDice(142))]
        );
    }

    #[test]
    fn read_request_with_id() {
        let requests = &NumericSession::from_str(&String::from("(FIRE +5)"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(1, NumericDice::ConstDice(5)).add_id(Some(String::from("FIRE")))]
        );

        let requests = &FudgeSession::from_str(&String::from("(ABC_4A 10F)"))
            .unwrap()
            .requests;
        assert_eq!(
            *requests,
            vec![RollRequest::new(10, FudgeDice::FudgeDice).add_id(Some(String::from("ABC_4A")))]
        );

        // Without parenthesis
        assert!(!&NumericSession::from_str(&String::from("FIRE +5")).is_ok());

        // Uncompliant ID
        assert!(!&NumericSession::from_str(&String::from("FI +5")).is_ok());
        assert!(!&NumericSession::from_str(&String::from("42A +5")).is_ok());
        assert!(!&NumericSession::from_str(&String::from("A2A +5")).is_ok());
        assert!(!&NumericSession::from_str(&String::from("_ABC +5")).is_ok());
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

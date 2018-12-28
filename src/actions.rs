use crate::dice::{DiceKind, RollResult};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ActionKind {
    Identity,
    FlipFlop,
    MultiplyBy(u16),
}

pub trait Transform {
    fn transform(&self, rolls: &Vec<RollResult>) -> Vec<RollResult>;
}

pub trait Aggregate {
    fn aggregate(rolls: &Vec<RollResult>) -> Option<RollResult>;
}

pub struct Identity;
impl Transform for Identity {
    fn transform(&self, rolls: &Vec<RollResult>) -> Vec<RollResult> {
        rolls.iter().map(|roll| roll.clone()).collect()
    }
}

/// Flip the digits of a numbered dice roll.
///
/// # Examples
/// - For a D20 roll : 1 -> 10, 15 -> 51, 20 -> 2
/// - For a D100 roll : 1 -> 100, 15 -> 510, 100 -> 1
pub struct FlipFlop;
impl Transform for FlipFlop {
    fn transform(&self, rolls: &Vec<RollResult>) -> Vec<RollResult> {
        return rolls
            .iter()
            .map(|roll| {
                // Compute the max padding required for 1 to become 10, 100, etc. according to the dice sides
                let max_dice = match roll.dice {
                    DiceKind::Mock(mock) => mock,
                    DiceKind::NumberedDice(sides) => sides,
                };
                let max_digits = get_digits_number(max_dice as f32);
                let result = format!("{:0width$}", roll.result, width = max_digits)
                    .chars()
                    .rev()
                    .collect::<String>();
                let result: u16 = result.parse().unwrap();
                RollResult {
                    // TODO : should the dice type change ?
                    dice: roll.dice,
                    result: result,
                }
            })
            .collect();
    }
}

fn get_digits_number(n: f32) -> usize {
    (f32::log10(n) + 1f32) as usize
}

pub struct MultiplyBy {
    factor: u16,
}
impl MultiplyBy {
    pub fn new(factor: u16) -> MultiplyBy {
        MultiplyBy { factor }
    }
}
impl Transform for MultiplyBy {
    fn transform(&self, rolls: &Vec<RollResult>) -> Vec<RollResult> {
        return rolls
            .iter()
            .map(|roll| RollResult {
                dice: roll.dice,
                result: roll.result * self.factor,
            })
            .collect();
    }
}

/// Return the sums of rolls for each different dice kind
///
/// To get the total sum regardless of dice kind, use [TotalSum](struct.TotalSum.html)
pub struct Sum;
impl Transform for Sum {
    fn transform(&self, rolls: &Vec<RollResult>) -> Vec<RollResult> {
        let mut sum_by_dice = HashMap::new();
        for roll in rolls.iter() {
            sum_by_dice
                .entry(roll.dice)
                .and_modify(|sum| *sum += roll.result)
                .or_insert(roll.result);
        }
        return sum_by_dice
            .iter()
            .map(|keyval| RollResult::new(*keyval.0, *keyval.1))
            .collect::<Vec<RollResult>>();
    }
}

/// Return a single sum of all rolls, regardless of dice kind
///
/// To get the sums of each kind of dice separately, use [Sum](struct.Sum.html)
pub struct TotalSum;
impl Aggregate for TotalSum {
    fn aggregate(rolls: &Vec<RollResult>) -> Option<RollResult> {
        // TODO change kind of dice to DiceKind::Aggregate?
        if rolls.len() == 0 {
            return None;
        }
        let result = rolls.iter().map(|roll| roll.result).sum();
        Some(RollResult {
            dice: rolls[0].dice,
            result: result,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::{ self, Aggregate, Transform};
    use crate::dice::{DiceKind, RollResult};

    static INPUT: &[RollResult] = &[
        RollResult {
            dice: DiceKind::NumberedDice(10),
            result: 1,
        },
        RollResult {
            dice: DiceKind::NumberedDice(10),
            result: 1,
        },
        RollResult {
            dice: DiceKind::NumberedDice(10),
            result: 1,
        },
        RollResult {
            dice: DiceKind::NumberedDice(20),
            result: 15,
        },
        RollResult {
            dice: DiceKind::NumberedDice(100),
            result: 100,
        },
    ];

    #[test]
    fn transform_identity() {
        let input = INPUT.to_vec();
        let output = actions::Identity {}.transform(&input);
        let expected = &input;
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i].dice, expected[i].dice);
            assert_eq!(output[i].result, expected[i].result);
        }
    }

    #[test]
    fn transform_multiply() {
        let input = INPUT.to_vec();
        let factor: u16 = 5;
        let output = actions::MultiplyBy::new(factor).transform(&input);
        let expected = &input;
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i].dice, expected[i].dice);
            assert_eq!(output[i].result, expected[i].result * 5);
        }
    }

    #[test]
    fn transform_flipflop() {
        let input = INPUT.to_vec();
        let output = actions::FlipFlop {}.transform(&input);
        let expected = vec![
            RollResult::new(DiceKind::NumberedDice(10), 10),
            RollResult::new(DiceKind::NumberedDice(10), 10),
            RollResult::new(DiceKind::NumberedDice(10), 10),
            RollResult::new(DiceKind::NumberedDice(20), 51),
            RollResult::new(DiceKind::NumberedDice(100), 2),
        ];
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i].dice, expected[i].dice);
            assert_eq!(output[i].result, expected[i].result);
        }
    }

    #[test]
    fn transform_sum() {
        let input = INPUT.to_vec();
        let output = actions::Sum {}.transform(&input);
        let expected = vec![
            RollResult::new(DiceKind::NumberedDice(10), 3),
            RollResult::new(DiceKind::NumberedDice(20), 15),
            RollResult::new(DiceKind::NumberedDice(100), 100),
        ];
        assert_eq!(output.len(), expected.len());
        for expected in expected.iter() {
            match output.iter().find(|sum| sum.dice == expected.dice) {
                None => assert!(false),
                Some(sum) => assert_eq!(sum.result, expected.result),
            }
        }
    }

    #[test]
    fn aggregate_total_sum() {
        let input = INPUT.to_vec();
        let output = actions::TotalSum::aggregate(&input);
        let expected = RollResult::new(DiceKind::NumberedDice(10), 118);

        match output {
            None => assert!(false, "Sum agregation should return a sum roll"),
            Some(output) => {
                assert_eq!(
                    output.dice, expected.dice,
                    "Sum should return the same rolls dice"
                );
                assert_eq!(
                    output.result, expected.result,
                    "Sum aggregation should sum the dice results"
                );
            }
        }
    }
}

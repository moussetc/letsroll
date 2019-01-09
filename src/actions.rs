//! `actions` is a collection of transformations that can be applied
//! to dice rolls : adding values, multiplying, computing the sum, etc.
//!
//! Some actions are only defined for a kind of roll (for example, you can
//! sum numeric rolls but not fudge rolls).

use crate::dice::AggregatedDice;
use crate::dice::AggregatedRoll;
use crate::dice::DiceKind;
use crate::dice::DiceNumber;
use crate::dice::GetMaxValue;
use crate::dice::Rolls;
use crate::dice::{FudgeRoll, NumericRoll, Roll};
use crate::errors::{Error, ErrorKind};
use crate::DiceRequest;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;

/// Enumeration of all possible actions
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Clone the rolls (cf. trait [Identity](trait.Identity.html)).
    Identity,
    /// TODO
    CountValues,
    /// Rerolls the dice for the values equal to the action parameter (numeric rolls only, cf. trait [Reroll](trait.Reroll.html)).
    RerollNumeric(NumericRoll),
    /// Rerolls the dice for the values equal to the action parameter (fudge rolls only, cf. trait [Reroll](trait.Reroll.html)).
    RerollFudge(FudgeRoll),
    /// Sum the rolls for each dice (numeric rolls only, cf. trait [Sum](trait.Sum.html)).
    Sum,
    // Sum all the dice (numeric rolls only, cf. trait [TotalSum](trait.TotalSum.html)).
    Total,
    /// Multiply the rolls by the action parameter (numeric rolls only, cf. trait [MultiplyBy](trait.MultiplyBy.html)).
    MultiplyBy(NumericRoll),
    /// Invert the digits of the rolls (numeric rolls only, cf. trait [FlipFlop](trait.FlipFlop.html)).   
    FlipFlop,
    /// Add new rolls for rolls equal to the highest value possible (numeric rolls only, cf. trait [Explode](trait.Explode.html)).   
    Explode(NumericRoll),
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Action that return a clone of the rolls
/// # Example
/// ```
/// # use letsroll::actions::Identity;
/// let input_rolls = vec![1,2,3];
/// assert_eq!(input_rolls.clone_rolls(), vec![1,2,3]);
/// ```
pub trait Identity<T> {
    fn clone_rolls(&self) -> T;
}
impl Identity<Vec<NumericRoll>> for Vec<NumericRoll> {
    fn clone_rolls(&self) -> Vec<NumericRoll> {
        self.iter().map(|roll| roll.clone()).collect()
    }
}
impl Identity<Vec<FudgeRoll>> for Vec<FudgeRoll> {
    fn clone_rolls(&self) -> Vec<FudgeRoll> {
        self.iter().map(|roll| roll.clone()).collect()
    }
}

/// Action that multiply the rolls by a factor
/// # Example
/// ```
/// # use letsroll::actions::MultiplyBy;
/// let input_rolls = vec![1,2,3];
/// assert_eq!(input_rolls.multiply(100), vec![100,200,300]);
/// ```
pub trait MultiplyBy<T> {
    fn multiply(&self, factor: NumericRoll) -> T;
}
impl MultiplyBy<Vec<NumericRoll>> for Vec<NumericRoll> {
    fn multiply(&self, factor: NumericRoll) -> Vec<NumericRoll> {
        self.iter().map(|roll| roll * factor).collect()
    }
}

/// Replace the rolls equal to the given value by a new roll
/// # Example
/// ```
/// # use letsroll::actions::Reroll;
/// # use letsroll::dice::ConstDice;
/// let mut input_rolls = vec![1,2,3];
/// let dice = ConstDice::new(42);
/// assert_eq!(input_rolls.reroll(&dice, &3), vec![1,2,42]);
/// ```
// TODO should the new roll be suject to the same action ?
pub trait Reroll<T: PartialEq, V: Roll> {
    fn reroll(&mut self, dice: &V, t: &T) -> Vec<T>;
}
impl<T: PartialEq + Clone, V: Roll<RollResult = Vec<T>>> Reroll<T, V> for Vec<T> {
    fn reroll(&mut self, dice: &V, t: &T) -> Vec<T> {
        let mut new_rolls: Vec<T> = vec![];
        for roll in self.iter() {
            if roll == t {
                new_rolls.append(&mut dice.roll(1));
            } else {
                new_rolls.push(roll.clone());
            }
        }
        new_rolls
    }
}

/// Flip the digits of a numbered dice roll.
/// # Example
/// Let's simulate a D20 flipflop:
/// ```
/// # use letsroll::actions::FlipFlop;
/// # use letsroll::dice::ConstDice;
/// let mut input_rolls = vec![1,15,20];
/// let dice = ConstDice::new(20);
/// assert_eq!(input_rolls.flip(&dice), vec![10,51,2]);
/// ```
/// And now a D100 flipflop:
/// ```
/// # use letsroll::actions::FlipFlop;
/// # use letsroll::dice::ConstDice;
/// let mut input_rolls = vec![1,15,20];
/// let dice = ConstDice::new(100);
/// assert_eq!(input_rolls.flip(&dice), vec![100,510,20]);
/// ```
pub trait FlipFlop<T, V: Roll> {
    fn flip(&self, dice: &V) -> Vec<T>;
}
impl<V: Roll + GetMaxValue> FlipFlop<NumericRoll, V> for Vec<NumericRoll> {
    fn flip(&self, dice: &V) -> Vec<NumericRoll> {
        self.iter()
            .map(|roll| {
                // Compute the max padding required for 1 to become 10, 100, etc. according to the dice sides
                let max_digits = get_digits_number(dice.get_max_value() as f32);
                let result = format!("{:0width$}", roll, width = max_digits)
                    .chars()
                    .rev()
                    .collect::<String>();
                let result: NumericRoll = result.parse().unwrap();
                result
            })
            .collect()
    }
}
fn get_digits_number(n: f32) -> usize {
    (f32::log10(n) + 1f32) as usize
}

/// Return the sum of numeric rolls
///
/// # Example
/// ```
/// # use letsroll::actions::Sum;
/// let input_rolls = vec![1,2,3];
/// assert_eq!(input_rolls.sum(), vec![6]);
/// ```
///
/// # Remark
/// This kind of action is only applied to the results rolls of once dice. To get the total sum of all dice, see [TotalSum](traits.TotalSum.html)
pub trait Sum<T> {
    fn sum(&self) -> T;
}
impl Sum<Vec<NumericRoll>> for Vec<NumericRoll> {
    fn sum(&self) -> Vec<NumericRoll> {
        vec![self.iter().sum()]
    }
}

/// Explode rerolls the dice whenever the highest value is rolled.
/// The new rolls can also trigger an explosion.
///
/// # Example
/// ```
/// # use letsroll::actions::Explode;
/// # use letsroll::dice::ConstDice;
/// let input_rolls = vec![1, 2, 3];
/// let dice = ConstDice::new(4);
/// assert_eq!(
///     input_rolls.explode(&dice, &2),
///     vec![1,2,3,4]
/// );
/// ```
/// # Warning
/// Don't use on a [ConstDice](../dice/struct.ConstDice.html) result with the same ConstDice for rerolls: it would end in stack overflow since the highest value=only value will always be rerolled
pub trait Explode<T, V: Roll> {
    fn explode(&self, dice: &V, explosion_value: &T) -> Vec<T>;
}
impl<T: PartialEq + Copy + Debug, V: Roll<RollResult = Vec<T>>> Explode<T, V> for Vec<T> {
    fn explode(&self, dice: &V, explosion_value: &T) -> Vec<T> {
        if self.len() == 0 {
            return vec![];
        }

        let new_rolls: Vec<T> =
            dice.roll(self.iter().filter(|roll| *roll == explosion_value).count() as DiceNumber);

        let mut rolls = self.clone();
        rolls.append(&mut new_rolls.explode(dice, explosion_value));
        rolls
    }
}

/// Return a single sum of all rolls, regardless of dice kind
///
/// To get the sums of each kind of dice separately, use [Sum](trait.Sum.html)
pub trait TotalSum {
    fn total(&self, dice: &Vec<DiceRequest>) -> Result<Rolls, Error>;
}
impl TotalSum for Vec<Rolls> {
    fn total(&self, dice: &Vec<DiceRequest>) -> Result<Rolls, Error> {
        if self.len() == 0 {
            return Ok(Rolls::Aggregation(AggregatedRoll {
                value: String::from("No dice to total :("),
            }));
        }

        let num_rolls: Result<Vec<Vec<NumericRoll>>, Error> = self
            .iter()
            .map(|typed_rolls| match typed_rolls {
                Rolls::NumericRolls(num_rolls) => Ok(num_rolls.clone()),
                _ => Err(Error::new(ErrorKind::IncompatibleAction(String::from(
                    "Impossible to compute a sum for non numerical rolls.",
                )))),
            })
            .collect();

        match num_rolls {
            Ok(rolls_to_sum) => {
                let subresults: Vec<(NumericRoll, String)> = rolls_to_sum
                    .iter()
                    .enumerate()
                    .map(|(index, rolls)| {
                        (
                            rolls.iter().sum(),
                            dice.get(index).expect("argh").to_string(),
                        )
                    })
                    .collect();
                let result: NumericRoll = rolls_to_sum.iter().flatten().sum();
                Ok(Rolls::Aggregation(AggregatedRoll {
                    value: format!(
                        "{}\nDetail: {}",
                        result,
                        subresults
                            .iter()
                            .map(|d| format!("{}: {}", d.1, d.0))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                }))
            }
            Err(err) => Err(err),
        }
    }
}

/// TODO ???
pub trait CountValues<T> {
    fn count(&self) -> T;
}
impl<T: Hash + Eq> CountValues<Vec<NumericRoll>> for Vec<T> {
    fn count(&self) -> Vec<NumericRoll> {
        let mut set: HashMap<&T, NumericRoll> = HashMap::new();
        for roll in self.iter() {
            set.entry(roll).and_modify(|count| *count += 1).or_insert(0);
        }
        set.iter().map(|keyval| *keyval.1).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::*;
    use crate::dice::{ConstDice, NumberedDice, NumericRoll, RepeatingDice};

    static NUM_INPUT: &[NumericRoll] = &[1, 1, 1, 15, 100];

    #[test]
    fn transform_identity() {
        let input = NUM_INPUT.to_vec();
        let output = input.clone_rolls();
        let expected = &input;
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
    }

    #[test]
    fn transform_count_values() {
        //TODO
        // :( It's useless to return sums without the associated value! Argh.
    }

    #[test]
    fn transform_multiply() {
        let input = NUM_INPUT.to_vec();
        let factor: NumericRoll = 5;
        let output = input.multiply(factor);
        let expected = &input;
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i] * factor);
        }
    }

    #[test]
    fn transform_flipflop() {
        let input = NUM_INPUT.to_vec();
        let output = input.flip(&NumberedDice::new(100));
        let expected = vec![100, 100, 100, 510, 2];
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
    }

    #[test]
    fn transform_sum() {
        let input = NUM_INPUT.to_vec();
        let output = input.sum();
        let expected = vec![118];
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
    }

    #[test]
    fn transform_reroll_num() {
        let mut input = NUM_INPUT.to_vec();
        let output = input.reroll(&ConstDice::new(42), &1);
        let expected = vec![42, 42, 42, 15, 100];
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
    }

    #[test]
    fn transform_reroll_text() {
        let mut input = vec![' ', '+', '-', '+', '-'];
        let output = input.reroll(&ConstDice::new(' '), &'-');
        let expected = vec![' ', '+', ' ', '+', ' '];
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
    }

    #[test]
    fn transform_explode() {
        let input = vec![1, 2, 3, 2, 1];
        let dice = RepeatingDice::new(vec![1, 2]).unwrap();
        let output = input.explode(&dice, &2);
        let expected = vec![1, 2, 3, 2, 1, 1, 2, 1];
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
    }

    // #[test]
    // fn aggregate_total_sum() {
    //     let input = NUM_INPUT.to_vec();
    //     let output = actions::TotalSum::aggregate(&input);
    //     let expected = Rolls::new(DiceKind::NumberedDice(10), 118);

    //     match output {
    //         None => assert!(false, "Sum agregation should return a sum roll"),
    //         Some(output) => {
    //             assert_eq!(
    //                 output.dice, expected.dice,
    //                 "Sum should return the same rolls dice"
    //             );
    //             assert_eq!(
    //                 output.result, expected.result,
    //                 "Sum aggregation should sum the dice results"
    //             );
    //         }
    //     }
    // }
}

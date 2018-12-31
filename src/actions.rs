use crate::dice::GetNumericDiceParameter;
use crate::dice::{NumericRoll, Roll, TextRoll};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Identity,
    CountValues,
    RerollNumeric(NumericRoll),
    RerollText(TextRoll),
    Sum,
    MultiplyBy(NumericRoll),
    FlipFlop,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait Identity<T> {
    fn do_nothing(&self) -> T;
}

impl Identity<Vec<NumericRoll>> for Vec<NumericRoll> {
    fn do_nothing(&self) -> Vec<NumericRoll> {
        self.iter().map(|roll| roll.clone()).collect()
    }
}

impl Identity<Vec<TextRoll>> for Vec<TextRoll> {
    fn do_nothing(&self) -> Vec<TextRoll> {
        self.iter().map(|roll| roll.clone()).collect()
    }
}

pub trait MultiplyBy<T> {
    fn multiply(&self, factor: NumericRoll) -> T;
}

impl MultiplyBy<Vec<NumericRoll>> for Vec<NumericRoll> {
    fn multiply(&self, factor: NumericRoll) -> Vec<NumericRoll> {
        self.iter().map(|roll| roll * factor).collect()
    }
}

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

pub trait Reroll<T: PartialOrd + PartialEq, V: Roll> {
    fn reroll(&mut self, dice: &V, t: &T) -> Vec<T>;
}

impl<T: PartialOrd + PartialEq + Clone, V: Roll<RollResult = Vec<T>>> Reroll<T, V> for Vec<T> {
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

// pub trait Aggregate {
//     fn aggregate(rolls: &Vec<Rolls>) -> Option<Rolls>;
// }

/// Flip the digits of a numbered dice roll.
///
/// # Examples
/// - For a D20 roll : 1 -> 10, 15 -> 51, 20 -> 2
/// - For a D100 roll : 1 -> 100, 15 -> 510, 100 -> 1
pub trait FlipFlop<T, V: Roll + GetNumericDiceParameter> {
    fn flip(&self, dice: &V) -> Vec<T>;
}

impl<V: Roll + GetNumericDiceParameter> FlipFlop<NumericRoll, V> for Vec<NumericRoll> {
    fn flip(&self, dice: &V) -> Vec<NumericRoll> {
        self.iter()
            .map(|roll| {
                // Compute the max padding required for 1 to become 10, 100, etc. according to the dice sides
                let max_digits = get_digits_number(dice.get_numeric_param() as f32);
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

/// Return the sums of rolls for each different dice kind
///
/// To get the total sum regardless of dice kind, use [TotalSum](struct.TotalSum.html)
pub trait Sum<T> {
    fn sum(&self) -> T;
}
impl Sum<Vec<NumericRoll>> for Vec<NumericRoll> {
    fn sum(&self) -> Vec<NumericRoll> {
        vec![self.iter().sum()]
    }
}

// /// Return a single sum of all rolls, regardless of dice kind
// ///
// /// To get the sums of each kind of dice separately, use [Sum](struct.Sum.html)
// pub struct TotalSum;
// impl Aggregate for TotalSum {
//     fn aggregate(rolls: &Vec<Rolls>) -> Option<Rolls> {
//         // TODO change kind of dice to DiceKind::Aggregate?
//         if rolls.len() == 0 {
//             return None;
//         }
//         let result = rolls.iter().map(|roll| roll.result).sum();
//         Some(Rolls {
//             dice: rolls[0].dice,
//             result: result,
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use crate::actions::{FlipFlop, Identity, MultiplyBy, Sum};
    use crate::dice::{NumberedDice, NumericRoll};

    static NUM_INPUT: &[NumericRoll] = &[1, 1, 1, 15, 100];

    #[test]
    fn transform_identity() {
        let input = NUM_INPUT.to_vec();
        let output = input.do_nothing();
        let expected = &input;
        assert_eq!(output.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output[i], expected[i]);
        }
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

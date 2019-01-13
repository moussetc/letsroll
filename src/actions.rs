//! `actions` is a collection of transformations that can be applied
//! to dice rolls : adding values, multiplying, computing the sum, etc.
//!
//! Some actions are only defined for a kind of roll (for example, you can
//! sum numeric rolls but not fudge rolls).

use crate::dice::NumericRolls;
use crate::dice::*;
use core::fmt::Debug;
use core::fmt::Display;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// Enumeration of all possible actions
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Action {
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
    /// Add new rolls for rolls equal to the action parameter (numeric rolls only, cf. trait [Explode](trait.Explode.html)).   
    Explode(NumericRoll),
    /// Add new rolls for rolls equal to the action parameter (fudge rolls only, cf. trait [Explode](trait.Explode.html)).   
    ExplodeFudge(FudgeRoll),
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait Actionable {
    fn is_allowed(action: &Action) -> bool;
}

impl Actionable for NumericRolls {
    fn is_allowed(action: &Action) -> bool {
        match action {
            Action::RerollNumeric(_)
            | Action::Sum
            | Action::MultiplyBy(_)
            | Action::FlipFlop
            | Action::Explode(_)
            | Action::CountValues => true,
            _ => false,
        }
    }
}

impl Actionable for FudgeRolls {
    fn is_allowed(action: &Action) -> bool {
        match action {
            Action::RerollFudge(_) | Action::Explode(_) | Action::CountValues => true,
            _ => false,
        }
    }
}

impl Actionable for Vec<NumericRolls> {
    fn is_allowed(action: &Action) -> bool {
        match action {
            Action::Total => true,
            a if NumericRolls::is_allowed(a) => true,
            _ => false,
        }
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
impl MultiplyBy<NumericRolls> for NumericRolls {
    fn multiply(&self, factor: NumericRoll) -> NumericRolls {
        Rolls {
            description: format!("({}) x {}", &self.description, factor),
            dice_request: self.dice_request.clone(),
            rolls: self.rolls.multiply(factor),
        }
    }
}

/// Replace the rolls equal to the given value by a new roll
/// # Example
/// ```
/// # use letsroll::actions::Reroll;
/// # use letsroll::dice::{Dice, NumericRolls, NumericDice, NumericRollRequest};
/// let input_rolls = vec![5,1,10];
/// let dice = Dice::new();
/// let dice_request = NumericRollRequest::new(3, NumericDice::RepeatingDice(input_rolls));
/// let rolls = NumericRolls::new(dice_request, &dice);
/// assert_eq!(rolls.reroll(&dice, &1).rolls, vec![5,5,10]);
/// ```
pub trait Reroll<T: Debug, V: Clone + Debug> {
    fn reroll(&self, dice: &Roll<T, V>, t: &T) -> Rolls<T, V>;
}
impl<T: PartialEq + Debug + Display + Copy, V: Clone + Debug> Reroll<T, V> for Rolls<T, V> {
    // TODO should the new roll be suject to the same action ?
    fn reroll(&self, dice: &Roll<T, V>, t: &T) -> Rolls<T, V> {
        let mut new_rolls: Vec<T> = vec![];
        for roll in self.rolls.iter() {
            if roll == t {
                new_rolls.append(&mut dice.roll(1, &self.dice_request.dice));
            } else {
                new_rolls.push(*roll);
            }
        }
        Rolls {
            description: format!("{} Reroll({})", self.description, t),
            dice_request: self.dice_request.clone(),
            rolls: new_rolls,
        }
    }
}

/// Flip the digits of a numbered dice roll.
/// # Example
/// Let's simulate a D20 flipflop:
/// ```
/// # use letsroll::actions::FlipFlop;
/// # use letsroll::dice::{Dice, NumericRolls, NumericDice, NumericRollRequest};
/// let input_rolls = vec![1,15,20];
/// let dice_request = NumericRollRequest::new(3, NumericDice::RepeatingDice(input_rolls));
/// let rolls = NumericRolls::new(dice_request, &Dice::new());
/// assert_eq!(rolls.flip().rolls, vec![10,51,2]);
/// ```
/// And now a D100 flipflop:
/// ```
/// # use letsroll::actions::FlipFlop;
/// # use letsroll::dice::{Dice, NumericRolls, NumericDice, NumericRollRequest};
/// let input_rolls = vec![1,15,100];
/// let dice_request = NumericRollRequest::new(3, NumericDice::RepeatingDice(input_rolls));
/// let rolls = NumericRolls::new(dice_request, &Dice::new());
/// assert_eq!(rolls.flip().rolls, vec![100,510,1]);
/// ```
pub trait FlipFlop<T> {
    fn flip(&self) -> T;
}
impl FlipFlop<NumericRolls> for NumericRolls {
    fn flip(&self) -> NumericRolls {
        Rolls {
            description: format!("flip({})", &self.description),
            dice_request: self.dice_request.clone(),
            rolls: self
                .rolls
                .iter()
                .map(|roll| {
                    // Compute the max padding required for 1 to become 10, 100, etc. according to the dice sides
                    let max_digits =
                        get_digits_number(self.dice_request.dice.get_max_value() as f32);
                    let result = format!("{:0width$}", roll, width = max_digits)
                        .chars()
                        .rev()
                        .collect::<String>();
                    let result: NumericRoll = result.parse().unwrap();
                    result
                })
                .collect(),
        }
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
/// # use letsroll::dice::{Dice, NumericRolls, NumericDice, NumericRollRequest};
/// let dice_request = NumericRollRequest::new(3, NumericDice::ConstDice(10));
/// let rolls = NumericRolls::new(dice_request, &Dice::new());
/// assert_eq!(rolls.sum().rolls, vec![30]);
/// ```
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
impl Sum<NumericRolls> for NumericRolls {
    fn sum(&self) -> NumericRolls {
        Rolls {
            description: format!("sum({})", &self.description),
            dice_request: self.dice_request.clone(),
            rolls: self.rolls.sum(),
        }
    }
}

/// Explode rerolls the dice whenever the highest value is rolled.
/// The new rolls can also trigger an explosion.
///
/// # Example
/// ```
/// # use letsroll::actions::Explode;
/// # use letsroll::dice::{Dice, NumericRolls, NumericDice, NumericRollRequest};
/// let dice_request = NumericRollRequest::new(
///     5,
///     NumericDice::RepeatingDice(vec![1, 2, 3, 2, 1]),
/// );
/// let dice = Dice::new();
/// let rolls = NumericRolls::new(dice_request, &dice);
/// let expected = vec![1, 2, 3, 2, 1, 1, 2, 1];
/// assert_eq!(rolls.explode(&dice, &2).rolls, expected);
/// ```
/// # Warning
/// Don't use on a [ConstDice](../dice/struct.ConstDice.html) result with the same ConstDice for rerolls: it would end in stack overflow since the highest value=only value will always be rerolled
pub trait Explode<T: Debug, V: Debug + Clone> {
    fn explode(&self, dice: &Roll<T, V>, explosion_value: &T) -> Rolls<T, V>;
}

impl<T: Debug + Display + PartialEq + Clone, V: Debug + Clone> Explode<T, V> for Rolls<T, V> {
    fn explode(&self, dice: &Roll<T, V>, explosion_value: &T) -> Rolls<T, V> {
        Rolls {
            description: format!("{} explode({})", self.description, &explosion_value),
            dice_request: self.dice_request.clone(),
            rolls: explode(&self.rolls, dice, &self.dice_request.dice, explosion_value),
        }
    }
}

fn explode<T: Clone + PartialEq, V>(
    rolls: &Vec<T>,
    dice: &Roll<T, V>,
    dicekind: &V,
    explosion_value: &T,
) -> Vec<T> {
    let mut rolls = rolls.clone();
    if rolls.len() != 0 {
        let new_rolls = dice.roll(
            rolls.iter().filter(|roll| *roll == explosion_value).count() as DiceNumber,
            dicekind,
        );
        rolls.append(&mut explode(&new_rolls, dice, dicekind, explosion_value));
    }
    rolls
}

/// Return a single sum of all rolls, regardless of dice kind
///
/// To get the sums of each kind of dice separately, use [Sum](trait.Sum.html)
pub trait TotalSum {
    fn total(&self) -> NumericRolls;
}
impl TotalSum for Vec<NumericRolls> {
    fn total(&self) -> NumericRolls {
        // TODO ne pas recalculer les sous sommes à chaque fois...
        let description = format!(
            "Detailed rolls\t: {}\nTOTAL SUM \t",
            self.iter()
                .map(|roll| format!(
                    "({}:{})",
                    roll.description.clone(),
                    roll.rolls.iter().sum::<NumericRoll>()
                ))
                .collect::<Vec<String>>()
                .join(" + ")
        );
        let sum: NumericRoll = match self.len() {
            0 => 0,
            _ => self.iter().map(|roll| roll.rolls.clone()).flatten().sum(),
        };

        Rolls {
            dice_request: RollRequest::new(1, NumericDice::ConstDice(sum)),
            description,
            rolls: vec![sum],
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

    static NUM_INPUT: &[NumericRoll] = &[1, 1, 1, 15, 100];

    //TODO assert descriptions after actions

    #[test]
    fn transform_multiply() {
        let input = NUM_INPUT.to_vec();
        let factor: NumericRoll = 5;
        let expected = &input.clone();
        let rolls_result = NumericRolls::new(
            RollRequest::new(5, NumericDice::RepeatingDice(input)),
            &Dice::new(),
        );
        let output = rolls_result.multiply(factor);
        assert_eq!(output.rolls.len(), expected.len());
        for i in 0..expected.len() - 1 {
            assert_eq!(output.rolls[i], expected[i] * factor);
        }
    }

    #[test]
    fn transform_flipflop() {
        let input = NUM_INPUT.to_vec();
        let dice_request =
            NumericRollRequest::new(input.len() as DiceNumber, NumericDice::RepeatingDice(input));
        let rolls = NumericRolls::new(dice_request, &Dice::new());
        let output = rolls.flip();
        let expected = vec![100, 100, 100, 510, 1];
        assert_eq!(output.rolls, expected);
    }

    #[test]
    fn transform_sum() {
        let input = NUM_INPUT.to_vec();
        let dice_request =
            NumericRollRequest::new(input.len() as DiceNumber, NumericDice::RepeatingDice(input));
        let rolls = NumericRolls::new(dice_request, &Dice::new());
        let output = rolls.sum();
        let expected = vec![118];
        assert_eq!(output.rolls, expected);
    }

    #[test]
    fn transform_reroll_num() {
        let input = NUM_INPUT.to_vec();
        let dice_request =
            NumericRollRequest::new(input.len() as DiceNumber, NumericDice::RepeatingDice(input));
        let dice = Dice::new();
        let rolls = NumericRolls::new(dice_request, &dice);
        let output = rolls.reroll(&dice, &100);
        let expected = vec![1, 1, 1, 15, 1];
        assert_eq!(output.rolls, expected);
    }

    #[test]
    fn transform_reroll_fudge() {
        let input = vec![FudgeRoll::Blank, FudgeRoll::Plus, FudgeRoll::Minus];
        let dice_request =
            FudgeRollRequest::new(input.len() as DiceNumber, FudgeDice::RepeatingDice(input));
        let dice = Dice::new();
        let rolls = FudgeRolls::new(dice_request, &dice);
        let output = rolls.reroll(&dice, &FudgeRoll::Minus);
        let expected = vec![FudgeRoll::Blank, FudgeRoll::Plus, FudgeRoll::Blank];
        assert_eq!(output.rolls, expected);
    }

    #[test]
    fn transform_explode() {
        let input = vec![1, 2, 3, 2, 1];
        let dice_request =
            NumericRollRequest::new(input.len() as DiceNumber, NumericDice::RepeatingDice(input));
        let dice = Dice::new();
        let rolls = NumericRolls::new(dice_request, &dice);
        let output = rolls.explode(&dice, &2);
        let expected = vec![1, 2, 3, 2, 1, 1, 2, 1];
        assert_eq!(output.rolls, expected);
    }

    // #[test]
    // fn transform_count_values() {
    //     //TODO
    //     // :( It's useless to return sums without the associated value! Argh.
    // }

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

use crate::actions::Action;
use crate::actions::Apply;
use crate::errors::Error;
use core::fmt::Debug;
use core::fmt::Display;
use core::hash::Hash;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;

pub type DiceID = String;
pub type DiceNumber = u8;
/// Type of roll result for numbered dice (like D20)
pub type NumericRoll = u32;
// Type of roll result for fudge dice (fate)
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FudgeRoll {
    Plus,
    Minus,
    Blank,
}

pub trait RollBounds: Sized + Debug + Display + Clone + Copy + Hash + Eq {}
impl RollBounds for NumericRoll {}
impl RollBounds for FudgeRoll {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NumericDice {
    ConstDice(NumericRoll),
    NumberedDice(NumericRoll),
    RepeatingDice(Vec<NumericRoll>),
    AggregationResult,
}

impl NumericDice {
    pub fn get_max_value(&self) -> NumericRoll {
        match self {
            NumericDice::ConstDice(const_value) => *const_value,
            NumericDice::NumberedDice(sides) => *sides,
            NumericDice::RepeatingDice(repeating_values) => {
                *repeating_values.iter().max().unwrap_or(&0)
            }
            NumericDice::AggregationResult => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum FudgeDice {
    FudgeDice,
    ConstDice(FudgeRoll),
    RepeatingDice(Vec<FudgeRoll>),
}

pub trait DiceBounds: Sized + Debug + Display + Clone {}
impl DiceBounds for NumericDice {}
impl DiceBounds for FudgeDice {}

#[derive(Debug)]
pub struct DiceGenerator {
    rng_ref: RefCell<ThreadRng>,
}

pub trait Roll<T, V>
where
    T: RollBounds,
    V: DiceBounds,
{
    fn roll(&self, n: DiceNumber, dice: &V) -> Vec<T>;
}

impl Roll<NumericRoll, NumericDice> for DiceGenerator {
    fn roll(&self, n: DiceNumber, dice: &NumericDice) -> Vec<NumericRoll> {
        match dice {
            NumericDice::ConstDice(const_value) => self.roll_const_dice(n, const_value),
            NumericDice::NumberedDice(sides) => self.roll_numbered_dice(n, sides),
            NumericDice::RepeatingDice(repeating_values) => {
                self.roll_repeating(n, repeating_values)
            }
            NumericDice::AggregationResult => unimplemented!(),
        }
    }
}

impl Roll<FudgeRoll, FudgeDice> for DiceGenerator {
    fn roll(&self, n: DiceNumber, dice: &FudgeDice) -> Vec<FudgeRoll> {
        match dice {
            FudgeDice::ConstDice(const_value) => self.roll_const_dice(n, const_value),
            FudgeDice::FudgeDice => self.roll_fudge_dice(n),
            FudgeDice::RepeatingDice(repeating_values) => self.roll_repeating(n, repeating_values),
        }
    }
}

impl DiceGenerator {
    pub fn new() -> DiceGenerator {
        DiceGenerator {
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }

    pub fn roll_repeating<T: RollBounds>(
        &self,
        n: DiceNumber,
        repeating_values: &Vec<T>,
    ) -> Vec<T> {
        let mut repeat_values = repeating_values.clone();
        for _ in 0..(n as usize / repeating_values.len()) {
            repeat_values.append(&mut repeating_values.clone());
        }
        repeat_values[0..(n as usize)].to_vec()
    }

    pub fn roll_const_dice<T: RollBounds>(&self, n: DiceNumber, const_value: &T) -> Vec<T> {
        (1..n + 1).map(|_| const_value.clone()).collect()
    }

    pub fn roll_numbered_dice(&self, n: DiceNumber, sides: &NumericRoll) -> Vec<NumericRoll> {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1).map(|_| rng.gen_range(1, sides + 1)).collect()
    }

    pub fn roll_fudge_dice(&self, n: DiceNumber) -> Vec<FudgeRoll> {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1)
            .map(|_| match rng.gen_range(1, 4) {
                1 => FudgeRoll::Blank,
                2 => FudgeRoll::Plus,
                _ => FudgeRoll::Minus,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollRequest<T: DiceBounds> {
    pub(crate) number: DiceNumber,
    pub(crate) id: Option<DiceID>,
    pub(crate) dice: T,
    pub actions: Vec<Action>,
}

pub type NumericRollRequest = RollRequest<NumericDice>;
pub type FudgeRollRequest = RollRequest<FudgeDice>;

impl<T: DiceBounds> RollRequest<T> {
    pub fn new(number: DiceNumber, dice: T) -> RollRequest<T> {
        RollRequest {
            number,
            id: None,
            dice,
            actions: vec![],
        }
    }

    pub fn add_id(mut self, id: Option<DiceID>) -> RollRequest<T> {
        self.id = id;
        self
    }

    pub fn add_action(mut self, action: Action) -> RollRequest<T> {
        self.actions.push(action);
        self
    }

    pub fn add_actions(self, actions: Vec<Action>) -> RollRequest<T> {
        let mut self_mut = self;
        for action in actions.into_iter() {
            self_mut = self_mut.add_action(action);
        }
        self_mut
    }
}

impl<V: DiceBounds> RollRequest<V> {
    pub fn roll<T: RollBounds>(&self, dice: &Roll<T, V>) -> Result<Rolls<T, V>, Error>
    where
        Rolls<T, V>: Apply<T, V>,
    {
        let mut rolls = Rolls::<T, V>::new(self.clone(), dice);
        for action in self.actions.iter() {
            rolls = Apply::<T, V>::apply(&rolls, action, dice)?;
        }
        Ok(rolls)
    }
}

#[derive(Debug)]
pub struct Rolls<T: RollBounds, V: DiceBounds> {
    pub dice: V,
    pub description: String,
    pub rolls: Vec<T>,
}

impl<T: RollBounds, V: DiceBounds> Rolls<T, V> {
    pub fn new(dice_request: RollRequest<V>, dice: &Roll<T, V>) -> Rolls<T, V> {
        Rolls {
            description: dice_request.to_string(),
            rolls: dice.roll(dice_request.number, &dice_request.dice),
            dice: dice_request.dice,
        }
    }
}

pub type NumericRolls = Rolls<NumericRoll, NumericDice>;
pub type FudgeRolls = Rolls<FudgeRoll, FudgeDice>;

#[cfg(test)]
mod tests {
    use crate::dice::*;

    #[test]
    fn const_generation() {
        let dice = DiceGenerator::new();
        let const_value = 42;
        let roll_number = 5;
        let rolls = dice.roll(roll_number, &NumericDice::ConstDice(const_value));
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert_eq!(*roll, const_value);
        }

        let const_value = FudgeRoll::Blank;
        let roll_number = 2;
        let rolls = dice.roll(roll_number, &FudgeDice::ConstDice(const_value));
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert_eq!(*roll, const_value);
        }
    }

    #[test]
    fn numbered_dice_generation() {
        let dice = DiceGenerator::new();
        let dice_sides = 42;
        let roll_number = 5;
        let rolls = dice.roll(roll_number, &NumericDice::NumberedDice(dice_sides));
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert!(*roll > 0, "Numbered dice generator rolls should be > 0");
            assert!(
                *roll <= dice_sides,
                "Numbered dice generator rolls should be <= to the number of sides on the dice"
            );
        }
    }

    #[test]
    fn repeating_dice() {
        let dice = DiceGenerator::new();
        let repeating_values = vec![1, 2, 3, 4, 5];

        assert_eq!(
            dice.roll(0, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![]
        );
        assert_eq!(
            dice.roll(3, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![1, 2, 3]
        );
        assert_eq!(
            dice.roll(5, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            dice.roll(15, &NumericDice::RepeatingDice(repeating_values.clone())),
            vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5]
        );
    }
}

use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Type roll result from numbered dice
pub type DiceNumber = u8;
pub type NumericRoll = u16;
pub type TextRoll = char;

pub trait Roll {
    type RollResult;
    fn roll(&self, n: DiceNumber) -> Self::RollResult;
}

pub trait GetNumericDiceParameter {
    fn get_numeric_param(&self) -> NumericRoll;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DiceKind {
    NumericKind(NumericDice),
    TextKind(TextDice),
}

// impl DiceKind {
//     pub fn get_dice<T>(&self) -> Box<dyn Dice<RollResult = T>> {
//         match self {
//             DiceKind::NumericKind(num_dice) => match num_dice {
//                 NumericDice::Const(dice) => Box::new(dice) as Box<Dice>,
//                 NumericDice::NumberedDice(dice) => Box::new(dice) as Box<Dice>,
//             },
//             DiceKind::TextKind(text_dice) => match text_dice {
//                 TextDice::FudgeDice => Box::new(dice) as Box<Dice>,
//             },
//         }
//     }
// }

impl fmt::Display for DiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DiceKind::NumericKind(num_dice) => match num_dice {
                    NumericDice::Const(dice) => format!("Const{}", dice.const_value),
                    NumericDice::NumberedDice(dice) => format!("D{}", dice.sides),
                },
                DiceKind::TextKind(text_dice) => match text_dice {
                    TextDice::FudgeDice(_) => String::from("F"),
                    TextDice::Const(d) => d.const_value.to_string(),
                },
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NumericDice {
    Const(Const<NumericRoll>),
    NumberedDice(NumberedDice),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TextDice {
    FudgeDice(FudgeDice),
    Const(Const<TextRoll>),
}

impl Roll for NumericDice {
    type RollResult = Vec<NumericRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        match self {
            NumericDice::Const(dice) => dice.roll(n),
            NumericDice::NumberedDice(dice) => dice.roll(n),
        }
    }
}

impl Roll for TextDice {
    type RollResult = Vec<TextRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        match self {
            TextDice::FudgeDice(dice) => dice.roll(n),
            TextDice::Const(dice) => dice.roll(n),
        }
    }
}

pub enum Rolls {
    NumericRolls(Vec<NumericRoll>),
    TextRolls(Vec<TextRoll>),
}
impl fmt::Display for Rolls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rolls::NumericRolls(rolls) => rolls
                    .iter()
                    .map(|roll| roll.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                Rolls::TextRolls(rolls) => rolls
                    .iter()
                    .map(|roll| roll.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            }
        )
    }
}

/// Dice that always return the same value
#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Const<T: Debug + PartialEq + Eq + Hash> {
    pub(crate) const_value: T,
}

impl<T: Debug + PartialEq + Eq + Hash> Const<T> {
    pub fn new(const_value: T) -> Const<T> {
        Const { const_value }
    }
}

impl<T: Debug + PartialEq + Eq + Hash + Copy> Roll for Const<T> {
    type RollResult = Vec<T>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        (1..n + 1).map(|_| self.const_value).collect()
    }
}

impl GetNumericDiceParameter for Const<NumericRoll> {
    fn get_numeric_param(&self) -> NumericRoll {
        self.const_value
    }
}

#[derive(Debug)]
pub struct NumberedDice {
    sides: NumericRoll,
    rng_ref: RefCell<ThreadRng>,
}

impl NumberedDice {
    pub fn new(sides: NumericRoll) -> NumberedDice {
        NumberedDice {
            sides,
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }

    pub fn get_sides(&self) -> NumericRoll {
        self.sides
    }
}

impl GetNumericDiceParameter for NumberedDice {
    fn get_numeric_param(&self) -> NumericRoll {
        self.sides
    }
}

impl PartialEq for NumberedDice {
    fn eq(&self, other: &NumberedDice) -> bool {
        self.sides == other.sides
    }
}

impl Eq for NumberedDice {}

impl Roll for NumberedDice {
    type RollResult = Vec<NumericRoll>;

    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1)
            .map(|_| rng.gen_range(1, self.sides + 1))
            .collect()
    }
}

impl Hash for NumberedDice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sides.hash(state);
    }
}

#[derive(Debug)]
pub struct FudgeDice {
    rng_ref: RefCell<ThreadRng>,
}

impl FudgeDice {
    pub fn new() -> FudgeDice {
        FudgeDice {
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }
}

impl Roll for FudgeDice {
    type RollResult = Vec<TextRoll>;
    fn roll(&self, n: DiceNumber) -> Self::RollResult {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1)
            .map(|_| match rng.gen_range(1, 4) {
                1 => ' ',
                2 => '+',
                _ => '-',
            })
            .collect()
    }
}

impl PartialEq for FudgeDice {
    fn eq(&self, _: &FudgeDice) -> bool {
        true
    }
}

impl Eq for FudgeDice {}

impl Hash for FudgeDice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO : ugly but all FudgeDice are one and the same...
        1.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::{
        self, Const, DiceKind, FudgeDice, NumberedDice, NumericDice, Roll, TextDice,
    };

    #[test]
    fn dice_kind_comparison() {
        assert_eq!(
            NumericDice::Const(Const::new(10)),
            NumericDice::Const(Const::new(10))
        );
        assert_ne!(
            NumericDice::Const(Const::new(10)),
            NumericDice::Const(Const::new(20))
        );
        assert_eq!(
            NumericDice::NumberedDice(NumberedDice::new(10)),
            NumericDice::NumberedDice(NumberedDice::new(10))
        );
        assert_ne!(
            NumericDice::NumberedDice(NumberedDice::new(10)),
            NumericDice::NumberedDice(NumberedDice::new(30))
        );
        assert_ne!(
            NumericDice::NumberedDice(NumberedDice::new(10)),
            NumericDice::Const(Const::new(10))
        );
        assert_eq!(FudgeDice::new(), FudgeDice::new());
        assert_eq!(
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10)))
        );
        assert_ne!(
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
            DiceKind::NumericKind(NumericDice::Const(Const::new(10)))
        );
        assert_ne!(
            DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(10))),
            DiceKind::TextKind(TextDice::FudgeDice(FudgeDice::new()))
        );
    }

    #[test]
    fn const_generation() {
        let const_value = 42;
        let roll_number = 5;
        let gen = Const::new(const_value);
        let rolls = gen.roll(roll_number);
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert_eq!(*roll, const_value);
        }
    }

    #[test]
    fn numbered_dice_generation() {
        let dice_sides = 42;
        let roll_number = 5;
        let gen = dice::NumberedDice::new(dice_sides);
        let rolls = gen.roll(roll_number);
        assert_eq!(rolls.len(), roll_number as usize);
        for roll in rolls.iter() {
            assert!(*roll > 0, "Numbered dice generator rolls should be > 0");
            assert!(
                *roll <= dice_sides,
                "Numbered dice generator rolls should be <= to the number of sides on the dice"
            );
        }
    }
}

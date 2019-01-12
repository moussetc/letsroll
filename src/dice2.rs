use crate::dice::FudgeRoll;
use crate::dice::NumericRoll;
use core::cell::RefCell;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NumericDice {
    ConstDice(NumericRoll),
    NumberedDice(NumericRoll),
    RepeatingDice(Vec<NumericRoll>),
}

impl fmt::Display for NumericDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NumericDice::ConstDice(const_value) => const_value.to_string(),
                NumericDice::NumberedDice(sides) => format!("D{}", sides),
                NumericDice::RepeatingDice(repeat_values) => format!("[TODO {:?}]", repeat_values),
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FudgeDice {
    FudgeDice,
    ConstDice(FudgeRoll),
    RepeatingDice(Vec<FudgeRoll>),
}

impl fmt::Display for FudgeDice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FudgeDice::ConstDice(const_value) => const_value.to_string(),
                FudgeDice::FudgeDice => String::from("F"),
                FudgeDice::RepeatingDice(repeat_values) => format!("[TODO {:?}]", repeat_values),
            }
        )
    }
}

pub struct Dice {
    rng_ref: RefCell<ThreadRng>,
}

impl Dice {
    pub fn new() -> Dice {
        Dice {
            rng_ref: RefCell::new(rand::thread_rng()),
        }
    }

    pub fn roll_numeric_dice(&self, n: NumericRoll, dice: &NumericDice) -> Vec<NumericRoll> {
        match dice {
            NumericDice::ConstDice(const_value) => self.roll_const_dice(n, const_value),
            NumericDice::NumberedDice(sides) => self.roll_numbered_dice(n, sides),
            NumericDice::RepeatingDice(repeating_values) => {
                self.roll_repeating(n, repeating_values)
            }
        }
    }

    pub fn roll_repeating<T: Clone>(&self, n: NumericRoll, repeating_values: &Vec<T>) -> Vec<T> {
        let mut repeat_values = repeating_values.clone();
        for _ in 0..(n as usize / repeating_values.len()) {
            repeat_values.append(&mut repeating_values.clone());
        }
        repeat_values[0..(n as usize)].to_vec()
    }

    pub fn roll_const_dice<T: Clone>(&self, n: NumericRoll, const_value: &T) -> Vec<T> {
        (1..n + 1).map(|_| const_value.clone()).collect()
    }

    pub fn roll_numbered_dice(&self, n: NumericRoll, sides: &NumericRoll) -> Vec<NumericRoll> {
        let mut rng = self.rng_ref.borrow_mut();
        (1..n + 1).map(|_| rng.gen_range(1, sides + 1)).collect()
    }

    pub fn roll_fudgey_dice(&self, n: NumericRoll, dice: &FudgeDice) -> Vec<FudgeRoll> {
        match dice {
            FudgeDice::ConstDice(const_value) => self.roll_const_dice(n, const_value),
            FudgeDice::FudgeDice => self.roll_fudge_dice(n),
            FudgeDice::RepeatingDice(repeating_values) => self.roll_repeating(n, repeating_values),
        }
    }

    pub fn roll_fudge_dice(&self, n: NumericRoll) -> Vec<FudgeRoll> {
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

pub struct DiceRequest<T> {
    number: NumericRoll,
    dice: T,
}

impl<T: Display> fmt::Display for DiceRequest<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.number, self.dice.to_string())
    }
}

pub struct RollResults<T, V> {
    dice: DiceRequest<V>,
    description: String,
    rolls: Vec<T>,
}

impl RollResults<NumericRoll, NumericDice> {
    pub fn new(dice: DiceRequest<NumericDice>) -> RollResults<NumericRoll, NumericDice> {
        RollResults {
            description: dice.to_string(),
            dice,
            rolls: vec![],
        }
    }
}

impl RollResults<FudgeRoll, FudgeDice> {
    pub fn new(dice: DiceRequest<FudgeDice>) -> RollResults<FudgeRoll, FudgeDice> {
        RollResults {
            description: dice.to_string(),
            dice,
            rolls: vec![],
        }
    }
}

impl<T: Display, V> fmt::Display for RollResults<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.description,
            self.rolls
                .iter()
                .map(|roll| roll.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

pub struct RollSession<T, V> {
    rolls: Vec<RollResults<T, V>>,
    dice: Dice,
}

impl RollSession<NumericRoll, NumericDice> {
    pub fn new(dice: Vec<DiceRequest<NumericDice>>) -> RollSession<NumericRoll, NumericDice> {
        RollSession {
            dice: Dice::new(),
            rolls: dice
                .into_iter()
                .map(|dice_request| RollResults::<NumericRoll, NumericDice>::new(dice_request))
                .collect(),
        }
    }
}

impl RollSession<FudgeRoll, FudgeDice> {
    pub fn new(dice: Vec<DiceRequest<FudgeDice>>) -> RollSession<FudgeRoll, FudgeDice> {
        RollSession {
            dice: Dice::new(),
            rolls: dice
                .into_iter()
                .map(|dice_request| RollResults::<FudgeRoll, FudgeDice>::new(dice_request))
                .collect(),
        }
    }
}

pub trait Session {
    fn getResults(&self) -> String;
}

impl Session for RollSession<NumericRoll, NumericDice> {
    fn getResults(&self) -> String {
        self.rolls.iter().map(|roll| roll.to_string()).collect()
    }
}

impl Session for RollSession<FudgeRoll, FudgeDice> {
    fn getResults(&self) -> String {
        self.rolls.iter().map(|roll| roll.to_string()).collect()
    }
}

pub fn parse(input: &String) -> Box<dyn Session> {
    match input {
        x if x == "D6" => Box::new(RollSession::<NumericRoll, NumericDice>::new(vec![
            DiceRequest {
                number: 3,
                dice: NumericDice::ConstDice(42),
            },
        ])),
        x if x == "F" => Box::new(RollSession::<FudgeRoll, FudgeDice>::new(vec![
            DiceRequest {
                number: 3,
                dice: FudgeDice::ConstDice(FudgeRoll::Minus),
            },
        ])),
        _ => panic!("argh"),
    }
}

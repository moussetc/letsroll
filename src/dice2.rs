use crate::actions;
use crate::dice::DiceNumber;
use crate::dice::FudgeRoll;
use crate::dice::NumericRoll;
use crate::errors::Error;
use core::cell::RefCell;
use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NumericDice {
    ConstDice(NumericRoll),
    NumberedDice(NumericRoll),
    RepeatingDice(Vec<NumericRoll>),
}

impl NumericDice {
    pub fn get_max_value(&self) -> NumericRoll {
        match self {
            NumericDice::ConstDice(const_value) => *const_value,
            NumericDice::NumberedDice(sides) => *sides,
            NumericDice::RepeatingDice(repeating_values) => {
                *repeating_values.iter().max().unwrap_or(&0)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum FudgeDice {
    FudgeDice,
    ConstDice(FudgeRoll),
    RepeatingDice(Vec<FudgeRoll>),
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

#[derive(Clone)]
pub struct DiceRequest<T: Clone> {
    pub(crate) number: DiceNumber,
    pub(crate) dice: T,
}

impl<T: Clone> DiceRequest<T> {
    pub fn new(number: DiceNumber, dice: T) -> DiceRequest<T> {
        DiceRequest { number, dice }
    }
}

pub struct RollResults<T, V: Clone> {
    pub(crate) dice: DiceRequest<V>,
    pub(crate) description: String,
    pub(crate) rolls: Vec<T>,
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

pub struct RollSession<T, V: Clone> {
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
    fn get_results(&self) -> String;
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error>;
}

impl Session for RollSession<NumericRoll, NumericDice> {
    fn get_results(&self) -> String {
        self.rolls.iter().map(|roll| roll.to_string()).collect()
    }
    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        unimplemented!();
    }
}

impl Session for RollSession<FudgeRoll, FudgeDice> {
    fn get_results(&self) -> String {
        self.rolls.iter().map(|roll| roll.to_string()).collect()
    }

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        unimplemented!();
    }
}

pub struct FullRollSession {
    subsessions: Vec<Box<dyn Session>>,
}

impl FullRollSession {
    pub fn new(subsessions: Vec<Box<dyn Session>>) -> FullRollSession {
        FullRollSession { subsessions }
    }
}

impl Session for FullRollSession {
    fn get_results(&self) -> String {
        self.subsessions
            .iter()
            .map(|session| session.get_results())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn add_step(&mut self, action: actions::Action) -> Result<(), Error> {
        unimplemented!();
    }
}

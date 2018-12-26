use rand::rngs::ThreadRng;
use rand::Rng;

#[derive(Debug)]
pub struct Roll {
    pub dice: u16,
    pub result: u16,
}

pub trait Dice {
    fn generate(&mut self) -> Roll;
}

pub struct Mock {
    dice: u16,
}

impl Mock {
    fn new(dice: u16) -> Mock {
        Mock { dice }
    }
}

impl Dice for Mock {
    fn generate(&mut self) -> Roll {
        Roll {
            dice: self.dice.clone(),
            result: 1,
        }
    }
}

pub struct NumberedDice {
    dice: u16,
    rng: ThreadRng,
}

impl NumberedDice {
    pub fn new(dice: u16) -> NumberedDice {
        NumberedDice {
            dice,
            rng: rand::thread_rng(),
        }
    }
}

impl Dice for NumberedDice {
    fn generate(&mut self) -> Roll {
        Roll {
            dice: self.dice,
            result: self.rng.gen_range(0, self.dice),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generators;
    use crate::generators::Dice;

    #[test]
    fn mock_generation() {
        let dice_sides = 42;
        let mut gen = generators::Mock::new(dice_sides);
        let roll = gen.generate();
        assert_eq!(
            roll.dice, dice_sides,
            "Mock generator didn't use the correct number of dice sides"
        );
        assert_eq!(roll.result, 1, "Mock generator should always roll a 1");
    }

    #[test]
    fn numbered_dice_generation() {
        let dice_sides = 42;
        let mut gen = generators::NumberedDice::new(dice_sides);
        let roll = gen.generate();
        assert_eq!(
            roll.dice, dice_sides,
            "Numbered dice generator didn't use the correct number of dice sides"
        );
        assert!(
            roll.result > 0,
            "Numbered dice generator rolls should be > 0"
        );
        assert!(
            roll.result <= dice_sides,
            "Numbered dice generator rolls should be <= to the number of sides on the dice"
        );
    }
}

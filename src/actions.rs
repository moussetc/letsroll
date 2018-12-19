use crate::generators::Roll;

pub trait Transform {
    fn transform(rolls: Vec<Roll>) -> Vec<Roll>;
}

pub trait Aggregate {
    fn aggregate(rolls: Vec<Roll>) -> Roll;
}

pub struct Identity;
impl Transform for Identity {
    fn transform(rolls: Vec<Roll>) -> Vec<Roll> {
        return rolls;
    }
}

pub struct FlipFlop;
impl Transform for FlipFlop {
    fn transform(rolls: Vec<Roll>) -> Vec<Roll> {
        return rolls
            .iter()
            .map(|roll| {
                let result = roll.result.to_string().chars().rev().collect::<String>();
                let result: i16 = result.parse().unwrap();
                Roll {
                    dice: roll.dice.clone(),
                    result: result,
                }
            })
            .collect();
    }
}

pub struct Sum;
impl Aggregate for Sum {
    fn aggregate(rolls: Vec<Roll>) -> Roll {
        let dice = rolls
            .iter()
            .map(|roll| roll.dice.clone())
            .collect::<Vec<String>>()
            .join(" ");
        let result = rolls.iter().map(|roll| roll.result).sum();
        Roll {
            dice: dice,
            result: result,
        }
    }
}

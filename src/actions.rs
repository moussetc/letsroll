use crate::generators::RollResult;

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

pub struct FlipFlop;
impl Transform for FlipFlop {
    fn transform(&self, rolls: &Vec<RollResult>) -> Vec<RollResult> {
        return rolls
            .iter()
            .map(|roll| {
                let result = roll.result.to_string().chars().rev().collect::<String>();
                let result: u16 = result.parse().unwrap();
                RollResult {
                    dice: roll.dice.clone(),
                    result: result,
                }
            })
            .collect();
    }
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
                dice: roll.dice.clone(),
                result: roll.result * self.factor,
            })
            .collect();
    }
}

pub struct Sum;
impl Aggregate for Sum {
    fn aggregate(rolls: &Vec<RollResult>) -> Option<RollResult> {
        // let dice = rolls
        //     .iter()
        //     .map(|roll| roll.dice.to_string())
        //     .collect::<Vec<String>>()
        //     .join(" ");
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
    use crate::actions;
    use crate::actions::Aggregate;
    use crate::actions::Transform;
    use crate::generators::RollResult;

    #[test]
    fn transform_identity() {
        let input = vec![
            RollResult::new(20, 1),
            RollResult::new(20, 15),
            RollResult::new(20, 20),
        ];
        let output = actions::Identity {}.transform(&input);
        let expected = &input;
        assert_eq!(
            output.len(),
            expected.len(),
            "Transform should keep the number of dice"
        );
        for i in 0..expected.len() - 1 {
            assert_eq!(
                output[i].dice, expected[i].dice,
                "Identity should return the same rolls dice"
            );
            assert_eq!(
                output[i].result, expected[i].result,
                "Identity should return the same rolls result"
            );
        }
    }

    #[test]
    fn transform_multiply() {
        let input = vec![
            RollResult::new(20, 1),
            RollResult::new(20, 15),
            RollResult::new(20, 20),
        ];
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
        let input = vec![
            RollResult::new(20, 1),
            RollResult::new(20, 15),
            RollResult::new(20, 20),
        ];
        let output = actions::FlipFlop {}.transform(&input);
        // TODO : should it be 10, 51, 2 ?
        // TODO : should the dice type change ?
        let expected = vec![
            RollResult::new(20, 1),
            RollResult::new(20, 51),
            RollResult::new(20, 2),
        ];
        assert_eq!(
            output.len(),
            expected.len(),
            "Transform should keep the number of dice"
        );
        for i in 0..expected.len() - 1 {
            assert_eq!(
                output[i].dice, expected[i].dice,
                "FlipFlop should return the same rolls dice"
            );
            assert_eq!(
                output[i].result, expected[i].result,
                "FlipFlop should flip the numbers of a dice roll"
            );
        }
    }

    #[test]
    fn aggregate_sum() {
        let input = vec![
            RollResult::new(20, 1),
            RollResult::new(20, 15),
            RollResult::new(20, 20),
        ];
        let output = actions::Sum::aggregate(&input);
        let expected = RollResult::new(20, 36);
        match output {
            None => assert!(false, "Sum agregation should return a sum roll"),
            Some(output) => {
                // TODO : what should be the dice type ?
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

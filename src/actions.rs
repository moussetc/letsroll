use crate::generators::Roll;

pub trait Transform {
    fn transform(rolls: &Vec<Roll>) -> Vec<Roll>;
}

pub trait Aggregate {
    fn aggregate(rolls: &Vec<Roll>) -> Option<Roll>;
}

pub struct Identity;
impl Transform for Identity {
    fn transform(rolls: &Vec<Roll>) -> Vec<Roll> {
        rolls.iter().map(|roll| roll.clone()).collect()
    }
}

pub struct FlipFlop;
impl Transform for FlipFlop {
    fn transform(rolls: &Vec<Roll>) -> Vec<Roll> {
        return rolls
            .iter()
            .map(|roll| {
                let result = roll.result.to_string().chars().rev().collect::<String>();
                let result: u16 = result.parse().unwrap();
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
    fn aggregate(rolls: &Vec<Roll>) -> Option<Roll> {
        // let dice = rolls
        //     .iter()
        //     .map(|roll| roll.dice.to_string())
        //     .collect::<Vec<String>>()
        //     .join(" ");
        if rolls.len() == 0 {
            return None;
        }
        let result = rolls.iter().map(|roll| roll.result).sum();
        Some(Roll {
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
    use crate::generators::Roll;

    #[test]
    fn transform_identity() {
        let input = vec![Roll::new(20, 1), Roll::new(20, 15), Roll::new(20, 20)];
        let output = actions::Identity::transform(&input);
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
    fn transform_flipflop() {
        let input = vec![Roll::new(20, 1), Roll::new(20, 15), Roll::new(20, 20)];
        let output = actions::FlipFlop::transform(&input);
        // TODO : should it be 10, 51, 2 ?
        // TODO : should the dice type change ?
        let expected = vec![Roll::new(20, 1), Roll::new(20, 51), Roll::new(20, 2)];
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
        let input = vec![Roll::new(20, 1), Roll::new(20, 15), Roll::new(20, 20)];
        let output = actions::Sum::aggregate(&input);
        let expected = Roll::new(20, 36);
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

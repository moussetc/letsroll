pub mod actions;
pub mod errors;
pub mod generators;

use crate::actions::Aggregate;
use crate::actions::Transform;
use crate::errors::Error;
use crate::generators::Dice;

pub trait ApplyGenerator: Sized {
    fn apply_generators(request: &Vec<Self>) -> Result<Vec<generators::Roll>, Error>;
}

impl ApplyGenerator for u16 {
    fn apply_generators(request: &Vec<u16>) -> Result<Vec<generators::Roll>, Error> {
        Ok(request
            .iter()
            .map(|x| generators::NumberedDice::new(*x).generate())
            .collect::<Vec<generators::Roll>>())
    }
}

impl ApplyGenerator for String {
    fn apply_generators(request: &Vec<String>) -> Result<Vec<generators::Roll>, Error> {
        let request = request
            .into_iter()
            .map(|n| n.parse::<u16>())
            .collect::<Result<Vec<u16>, _>>()?;
        u16::apply_generators(&request)
    }
}

#[cfg(test)]
mod tests {
    use crate::errors;
    use crate::ApplyGenerator;

    #[test]
    fn numeric_request_generation() {
        let request = vec![10, 10, 10];
        let rolls = u16::apply_generators(&request);
        match rolls {
            Ok(rolls) => assert_eq!(
                rolls.len(),
                request.len(),
                "Generation should roll as many dice as requested (expected {})",
                request.len()
            ),
            Err(_) => assert!(false, "Valid request should be generated correctly"),
        }
    }

    #[test]
    fn good_string_generation() {
        let request = vec![String::from("10"), String::from("10"), String::from("10")];
        let rolls = String::apply_generators(&request);
        match rolls {
            Ok(rolls) => assert_eq!(
                rolls.len(),
                request.len(),
                "Generation should roll as many dice as requested (expected {})",
                request.len(),
            ),
            Err(_) => assert!(false, "Valid request should be parsed correctly"),
        }
    }

    #[test]
    fn bad_string_generation() {
        let request = vec![
            String::from("qsdqsd"),
            String::from("qds"),
            String::from("qds"),
        ];
        let rolls = String::apply_generators(&request);
        match rolls {
            Ok(_) => assert!(false, "Invalid request should not be parsed correctly"),
            Err(_) => assert!(true, "Invalid request should result in error"),
        }
    }
}

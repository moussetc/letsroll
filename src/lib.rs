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
        let rolls: Vec<generators::Roll> = request
            .iter()
            .map(|x| generators::NumberedDice::new(*x).generate())
            .collect();

        Ok(vec![actions::Sum::aggregate(actions::FlipFlop::transform(
            rolls,
        ))])
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
// #[cfg(test)]
// mod tests {
//     #[test]
//     fn roll_classicGen() {
//         let gen = generators::NumberedDice::new(20);
//         gen.generate()
//     }
// }

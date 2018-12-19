pub mod actions;
pub mod generators;

use crate::actions::Aggregate;
use crate::actions::Transform;
use crate::generators::Generator;

pub fn apply_generators(request: &Vec<String>) -> Vec<generators::Roll> {
    let rolls: Vec<generators::Roll> = request
        .iter()
        .map(|x| generators::ClassicGen::generate(x))
        .collect();

    vec![actions::Sum::aggregate(actions::FlipFlop::transform(rolls))]
}

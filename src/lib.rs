mod generators;

use crate::generators::Generator;

pub fn apply_generators(request: &Vec<String>) -> Vec<generators::Roll> {
    request
        .iter()
        .map(|x| generators::ClassicGen::generate(x))
        .collect()
}

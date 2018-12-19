use rand::Rng;

#[derive(Debug)]
pub struct Roll {
    dice: String,
    result: i16,
}

pub trait Generator {
    // Define a method on the caller type which takes an
    // additional single parameter `T` and does nothing with it.
    fn generate(dice: &String) -> Roll;
}

struct Mock;
impl Generator for Mock {
    fn generate(a: &String) -> Roll {
        Roll {
            dice: "D6".to_string(),
            result: 3,
        }
    }
}

pub struct ClassicGen;
impl Generator for ClassicGen {
    fn generate(a: &String) -> Roll {
        let sides = a.parse::<i16>().unwrap();
        let result: i16 = rand::thread_rng().gen_range(0, sides);

        Roll {
            dice: "TODO".to_string(),
            result: result,
        }
    }
}

use letsroll::{self, Action};
use std::str::FromStr;

fn main() {
    // let request =
    //     letsroll::RollRequest::new(vec![DiceRequest::new(dice::DiceKind::NumberedDice(20), 5)]);
    // println!("{}", request);

    let dice_request = String::from("5D8 4D2 +42");
    let mut request = letsroll::RollRequest::from_str(&dice_request);
    match request {
        Err(msg) => println!("FAILURE : {}", msg),
        Ok(ref mut req) => {
            println!("{}", req);
            match req.add_step(Action::MultiplyBy(100)) {
                Err(msg) => println!("FAILURE : {}", msg),
                Ok(_) => println!("{}", req),
            }
        }
    }
}

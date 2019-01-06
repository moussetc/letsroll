use letsroll::{self, Action, DiceRequest};
use letsroll::dice::{DiceKind, NumericDice, NumberedDice};
use std::str::FromStr;

fn main() {
    let request =
        letsroll::RollRequest::new(vec![DiceRequest::new(DiceKind::NumericKind(NumericDice::NumberedDice(NumberedDice::new(20))), 5)]);
    println!("{}", request);

    let parsed_request = String::from("5D8 4D2 +42");
    let mut request = letsroll::RollRequest::from_str(&parsed_request);
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

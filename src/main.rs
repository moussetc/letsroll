use letsroll;
use std::str::FromStr;

fn main() {
    // let request =
    //     letsroll::RollRequest::new(vec![DiceRequest::new(dice::DiceKind::NumberedDice(20), 5)]);
    // println!("{}", request);

    let dice_request = String::from("5D8 4D2 3F");
    let request = letsroll::RollRequest::from_str(&dice_request);
    match request {
        Err(msg) => println!("FAILURE : {}", msg),
        Ok(req) => println!("{}", req),
    }
}

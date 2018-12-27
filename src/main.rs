use letsroll;
use letsroll::generators;
use letsroll::ApplyGenerator;
use letsroll::DiceRequest;
use std::str::FromStr;

fn main() {
    println!("Hello, world!");

    let request = vec![String::from("20"), String::from("4")];
    let rolls = String::apply_generators(&request);
    println!("{:?}", rolls);

    let request = letsroll::RollRequest::new(vec![DiceRequest::new(
        generators::DiceKind::NumberedDice(20),
        5,
    )]);
    println!("{}", request);

    let dice_request = String::from("5D8 4D2");
    let request = letsroll::RollRequest::from_str(&dice_request);
    match request {
        Err(msg) => println!("FAILURE : {}", msg),
        Ok(req) => println!("{}", req),
    }
}

use letsroll::dice::{NumericDice, RollRequest};
use letsroll::NumericSession;

fn main() {
    let dice_request = vec![
        RollRequest::new(4, NumericDice::NumberedDice(20)),
        RollRequest::new(2, NumericDice::NumberedDice(6)),
    ];

    let request = NumericSession::build(dice_request);
    println!("{}", request.to_string());
}

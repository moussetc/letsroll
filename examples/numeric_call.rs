use letsroll::dice::{NumericDice, NumericRoll, RollRequest};
use letsroll::{NumericSession, Session};

fn main() {
    let dice_request = vec![
        RollRequest::new(4, NumericDice::NumberedDice(20)),
        RollRequest::new(2, NumericDice::NumberedDice(6)),
    ];

    let request = NumericSession::new(dice_request);
    println!("{}", request.get_results());
}

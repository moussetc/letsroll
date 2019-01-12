use letsroll::dice::{DiceRequest, NumericDice, NumericRoll};
use letsroll::{RollSession, Session};

fn main() {
    let dice_request = vec![
        DiceRequest::new(4, NumericDice::NumberedDice(20)),
        DiceRequest::new(2, NumericDice::NumberedDice(6)),
    ];

    let request = RollSession::<NumericRoll, NumericDice>::new(dice_request);
    println!("{}", request.get_results());
}

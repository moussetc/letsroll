use letsroll;
use letsroll::dice::{DiceRequest, FudgeDice, FudgeRoll, RollSession, Session};

fn main() {
    let dice_request = vec![DiceRequest::new(4, FudgeDice::FudgeDice)];

    let request = RollSession::<FudgeRoll, FudgeDice>::new(dice_request);

    println!("{}", request.get_results());
}

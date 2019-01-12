use letsroll::dice::{FudgeDice, FudgeRoll, RollRequest};
use letsroll::{FudgeSession, Session};

fn main() {
    let dice_request = vec![RollRequest::new(4, FudgeDice::FudgeDice)];

    let request = FudgeSession::new(dice_request);

    println!("{}", request.get_results());
}

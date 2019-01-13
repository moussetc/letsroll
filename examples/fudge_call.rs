use letsroll::dice::{FudgeDice, RollRequest};
use letsroll::{FudgeSession, Session};

fn main() {
    let dice_request = vec![RollRequest::new(4, FudgeDice::FudgeDice)];

    let request = FudgeSession::new(dice_request);

    println!("{}", request.to_string());
}

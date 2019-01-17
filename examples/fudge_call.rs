use letsroll::dice::{FudgeDice, RollRequest};
use letsroll::FudgeSession;

fn main() {
    let dice_request = vec![RollRequest::new(4, FudgeDice::FudgeDice)];

    let request = FudgeSession::build(dice_request);

    println!("{}", request.to_string());
}

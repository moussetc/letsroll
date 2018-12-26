use letsroll::ApplyGenerator;

fn main() {
    println!("Hello, world!");

    let request = vec![String::from("20"), String::from("4")];

    let rolls = String::apply_generators(&request);
    println!("{:?}", rolls)
}

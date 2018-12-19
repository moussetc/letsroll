fn main() {
    println!("Hello, world!");

    let request = vec![String::from("20"), String::from("4")];

    let rolls = letsroll::apply_generators(&request);

    println!("{:?}", rolls)
}

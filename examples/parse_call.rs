use letsroll::Session;

fn main() {
    let parsed_request = String::from("5D8 4D2");
    let mut request = letsroll::io::read::parse_request(&parsed_request);
    match request {
        Err(msg) => println!("FAILURE : {}", msg),
        Ok(ref mut req) => {
            println!("{}", req.get_results());
        }
    }
}

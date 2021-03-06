use letsroll::io::read;

fn main() {
    let parsed_request = String::from("5D8 4D2 +1000");
    let mut request = read::parse_request(&parsed_request, false);
    match request {
        Err(msg) => eprintln!("FAILURE : {}", msg),
        Ok(ref mut req) => {
            println!("{}", req.to_string());
        }
    }
}

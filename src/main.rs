#[macro_use]
extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;
use letsroll::Session;

use letsroll::errors::Error;

use letsroll;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
// Write the Docopt usage string.
const USAGE: &'static str = "
Usage: letsroll <dice> [-s <savepath>]
       letsroll -f <filename> [-s <savepath>]
       letsroll (-h | --help)

Options:
    -h --help    Show this screen.
    -f, --file   Read the dice request from a file.
    -s, --save   Saves the rolls request to a file for future use. Tip: use .roll file extension!
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_dice: String,
    arg_filename: Option<String>,
    arg_savepath: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    match run(args) {
        Ok(_) => (),
        Err(error) => {
            println!("FAILURE : {}", error);
        }
    }
}

fn run(args: Args) -> Result<(), Error> {
    let request_to_parse = match &args.arg_filename {
        Some(filename) => match fs::read_to_string(filename) {
            Err(msg) => return Err(Error::from(msg)),
            Ok(file_contents) => file_contents,
        },
        None => args.arg_dice,
    };

    let roll_sessions = letsroll::io::read::parse_request(&request_to_parse, true);
    match roll_sessions {
        Err(msg) => return Err(msg),
        Ok(ref req) => {
            println!("Rolling...\n{}", req.to_string());
            match &args.arg_savepath {
                Some(save_path) => match write_to_file(&request_to_parse, &save_path) {
                    Ok(_) => {
                        println!("Wrote rolls request to file {}", save_path);
                        Ok(())
                    }
                    Err(msg) => return Err(Error::from(msg)),
                },
                _ => Ok(()),
            }
            }
        }
}

fn write_to_file(content: &String, filepath: &str) -> std::io::Result<()> {
        let path = Path::new(filepath);

        let mut file = File::create(&path)?;
        file.write_all(content.to_string().as_bytes())
    }
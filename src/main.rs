#[macro_use]
extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;
use letsroll::errors::Error;
use letsroll::RollRequest;

use letsroll;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

// Write the Docopt usage string.
const USAGE: &'static str = "
Usage: letsroll <dice> [-s <savepath>]
       letsroll -f <filename> [-s <savepath>]
       letsroll (-h | --help)

Options:
    -h --help    Show this screen.
    -f, --file   Read the dice request from a file.
    -s, --save   Saves the results in a file.
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

    let mut request = letsroll::RollRequest::from_str(&request_to_parse);

    match request {
        Err(msg) => return Err(msg),
        Ok(ref mut req) => {
            println!("Rolling...\n{}", req);
            match &args.arg_savepath {
                Some(save_path) => match write_results_to_file(&save_path, req) {
                    Ok(_) => {
                        println!("Wrote results to file {}", save_path);
                        Ok(())
                    }
                    Err(msg) => return Err(Error::from(msg)),
                },
                _ => Ok(()),
            }
        }
    }
}

fn write_results_to_file(filepath: &str, results: &RollRequest) -> std::io::Result<()> {
    let path = Path::new(filepath);

    let mut file = File::create(&path)?;
    file.write_all(results.to_string().as_bytes())
}

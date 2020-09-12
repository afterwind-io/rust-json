use colored::*;
use std::fs;
use std::path::PathBuf;

use super::utils::{read_file_as_utf8, UTF8Reader};
use super::validator::validate;

pub fn run_suite() {
    let entries: Vec<PathBuf> = fs::read_dir("JSONTestSuite/test_parsing")
        .unwrap()
        .map(|res| res.unwrap())
        .map(|res| res.path())
        .collect();

    for entry in entries {
        let filename = entry.file_name().unwrap();

        match read_file_as_utf8(&entry) {
            Err(x) => {
                println!("{:?} {:?}\n", filename, x);
                continue;
            }
            Ok(document) => {
                let reader = UTF8Reader::new(&document);
                let expect = &filename.to_str().unwrap()[0..1];

                let result = validate(&reader);
                println!(
                    "[{}] {:?}\n",
                    match result {
                        Ok(_) if expect == "y" => "Pass".bright_green(),
                        Err(_) if expect == "n" => "Pass".bright_green(),
                        _ if expect == "i" => "Pass".bright_green(),
                        _ => "Fail".bright_red(),
                    },
                    filename
                );
                println!(
                    "{}\n\n{}\n------------------------",
                    document.bright_yellow(),
                    match result {
                        Err(reason) => reason,
                        _ => String::default(),
                    }
                );
            }
        }
    }
}

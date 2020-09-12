use std::path::PathBuf;

mod validator;
use validator::validate;

mod test;
use test::run_suite;

mod utils;
use utils::{read_file_as_utf8, UTF8Reader};

fn main() {
    run_suite();

    // let content = read_file_as_utf8(&PathBuf::from(
    //     "JSONTestSuite/test_parsing/y_object_empty.json",
    //     "test/playground.json",
    // ))
    // .unwrap();
    // let reader = UTF8Reader::new(&content);
    // println!("{:?}", validate(&reader));
    // println!("{:?}", reader.get_tail());
}

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

pub fn read_file_as_utf8(entry: &PathBuf) -> Result<String, ()> {
    let path = entry.to_str().unwrap();

    let mut file = match File::open(path) {
        Err(why) => {
            println!("couldn't open. {}", why);
            return Err(());
        }
        Ok(file) => file,
    };

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(why) => {
            println!("couldn't read. {}", why);
            return Err(());
        }
        Ok(_) => {}
    }

    return Ok(content);
}

pub enum UTF8ReaderResult<'a> {
    Ok(&'a str),
    OutOfBoundError(usize),
}

impl<'a> UTF8ReaderResult<'a> {
    pub fn unwrap(self) -> &'a str {
        match self {
            UTF8ReaderResult::Ok(s) => s,
            UTF8ReaderResult::OutOfBoundError(_) => panic!("Index out of bound"),
        }
    }
}

pub struct UTF8Reader<'a> {
    document: &'a str,
    begin_index_map: Vec<usize>,
}

impl<'a> UTF8Reader<'a> {
    pub fn look_ahead(&self, index: usize, width: usize) -> UTF8ReaderResult {
        let l = self.len();

        let end_index = index + width;
        if end_index > l {
            return UTF8ReaderResult::OutOfBoundError(l - index);
        }

        let begin = self.begin_index_map[index];
        let end = self.begin_index_map[index + width];

        return UTF8ReaderResult::Ok(&self.document[begin..end]);
    }

    pub fn len(&self) -> usize {
        return self.begin_index_map.len() - 1;
    }

    pub fn new(document: &'a str) -> Self {
        let graphemes = UnicodeSegmentation::graphemes(document, true).collect::<Vec<&str>>();

        let mut sum = 0;
        let mut begin_index_map = graphemes
            .iter()
            .map(|g| {
                let s = sum;
                sum += g.len();
                return s;
            })
            .collect::<Vec<usize>>();
        begin_index_map.push(sum);

        return UTF8Reader {
            document,
            begin_index_map,
        };
    }
}

use crate::{DocID, InvertedIndex, PostingList};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::Regex;
use std::io::BufRead;

#[derive(Clone, Debug)]
pub struct SimpleInvertedIndex {
    docs: Vec<String>,
    index: HashMap<String, PostingList>,
}

fn tokenize_file(path: &Path) -> impl Iterator<Item = String> {
    lazy_static! {
        static ref WORD_REGEX: Regex = Regex::new(r"\b\w+['-]*\w*\b").unwrap();
    }

    let file_handle = File::open(&path).expect(&format!("Wrong path to file {path:?}"));
    let lines = std::io::BufReader::new(file_handle)
        .lines()
        .map(|l| l.unwrap());

    return lines.flat_map(|l| {
        WORD_REGEX
            .find_iter(&l)
            .map(|m| m.as_str().to_lowercase())
            .collect::<Vec<String>>()
            .into_iter()
    });
}

fn process_file(docs: &mut Vec<String>, index: &mut HashMap<String, PostingList>, file: &Path) {
    let doc_id = docs.len() as DocID;
    docs.push(file.to_str().unwrap().to_owned());

    for word in tokenize_file(file) {
        index
            .entry(word)
            .and_modify(|list| list.push(doc_id))
            .or_insert(vec![doc_id]);
    }
}

impl InvertedIndex for SimpleInvertedIndex {
    fn new() -> Self {
        Self {
            docs: Vec::new(),
            index: HashMap::new(),
        }
    }

    fn build(paths: Vec<PathBuf>, _num_threads: i32) -> Self {
        let mut docs = Vec::new();
        let mut index = HashMap::new();

        for path in paths {
            if path.is_file() {
                process_file(&mut docs, &mut index, &path);
            } else if path.is_dir() {
                for entry in walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        process_file(&mut docs, &mut index, entry.path());
                    }
                }
            }
        }

        Self { docs, index }
    }

    fn get(&self, term: &str) -> Vec<String> {
        self.index
            .get(term)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|id| self.docs[*id as usize].to_string())
            .collect()
    }
}

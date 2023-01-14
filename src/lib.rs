use std::{
    collections::HashMap,
    fs::{File, FileType},
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, BufRead};
use tokio::fs;
use walkdir::WalkDir;

pub mod rpc;

type DocID = i32;
type PostingList = Vec<DocID>;

#[derive(Clone, Debug)]
pub struct InvertedIndex {
    docs: Vec<String>,
    index: HashMap<String, PostingList>,
}

fn process_file(docs: &mut Vec<String>, index: &mut HashMap<String, PostingList>, file: &Path) {
    lazy_static! {
        static ref WORD_REGEX: Regex = Regex::new(r"\b\w+['-]*\w*\b").unwrap();
    }

    let file_handle = File::open(&file).expect(&format!("Wrong path to file {file:?}"));
    let lines = std::io::BufReader::new(file_handle)
        .lines()
        .map(|l| l.unwrap());

    let words = lines.flat_map(|l| {
        WORD_REGEX
            .find_iter(&l)
            .map(|m| m.as_str().to_lowercase())
            .collect::<Vec<String>>()
            .into_iter()
    });

    let doc_id = docs.len() as DocID;
    docs.push(file.to_str().unwrap().to_owned());

    for word in words {
        index
            .entry(word)
            .and_modify(|list| list.push(doc_id))
            .or_insert(vec![doc_id]);
    }
}

impl InvertedIndex {
    pub fn new() -> InvertedIndex {
        InvertedIndex {
            docs: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn build(paths: Vec<PathBuf>, _num_threads: i32) -> InvertedIndex {
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

        InvertedIndex { docs, index }
    }
    pub fn get(&self, term: &str) -> Vec<String> {
        self.index
            .get(term)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|id| self.docs[*id as usize].to_string())
            .collect()
    }
}

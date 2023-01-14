use std::path::PathBuf;

pub mod rpc;
pub mod simple_inverted_index;

type DocID = i32;
type PostingList = Vec<DocID>;

pub trait InvertedIndex {
    fn new() -> Self;
    fn build(paths: Vec<PathBuf>, num_threads: i32) -> Self;
    fn get(&self, term: &str) -> Vec<String>;
}

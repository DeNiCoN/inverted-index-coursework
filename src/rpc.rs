#[tarpc::service]
pub trait InvertedIndexService {
    async fn build(paths: Vec<String>);
    async fn get(term: String) -> Vec<String>;
}

use std::{path::PathBuf, sync::Arc};

use futures::future::{self, Ready};
use inverted_index_coursework::{
    rpc::{InvertedIndexService, InvertedIndexServiceClient},
    simple_inverted_index::{
        MultiFileThreadedSimpleInvertedIndex, SimpleInvertedIndex, ThreadedSimpleInvertedIndex,
    },
    InvertedIndex,
};
use tarpc::{
    client, context,
    server::{self, Channel},
};
use tokio::sync::RwLock;

#[derive(Clone)]
struct Server<T: InvertedIndex> {
    inverted_index_lock: Arc<RwLock<T>>,
}

impl<T: InvertedIndex> Server<T> {
    fn new() -> Self {
        Self {
            inverted_index_lock: Arc::new(RwLock::new(T::new())),
        }
    }
}

#[tarpc::server]
impl<T: InvertedIndex + std::marker::Send + std::marker::Sync + 'static> InvertedIndexService
    for Server<T>
{
    async fn get(self, _: context::Context, term: String) -> Vec<String> {
        self.inverted_index_lock.read().await.get(&term)
    }

    async fn build(self, _: context::Context, paths: Vec<String>) {
        let mut inverted_index = self.inverted_index_lock.write().await;
        *inverted_index =
            InvertedIndex::build(paths.into_iter().map(|s| PathBuf::from(s)).collect(), 12);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    let server_data = Server::<MultiFileThreadedSimpleInvertedIndex>::new();

    let server = server::BaseChannel::with_defaults(server_transport);
    tokio::spawn(server.execute(server_data.serve()));

    let client =
        InvertedIndexServiceClient::new(client::Config::default(), client_transport).spawn();

    client
        .build(
            context::current(),
            vec!["./data/datasets/aclImdb/train/unsup".to_string()],
        )
        .await?;

    let hello = client
        .get(context::current(), "manpower".to_string())
        .await?;

    println!("{hello:?}");

    Ok(())
}

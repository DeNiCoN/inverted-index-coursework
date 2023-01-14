use std::{path::PathBuf, sync::Arc};

use futures::future::{self, Ready};
use inverted_index_coursework::{
    rpc::{InvertedIndexService, InvertedIndexServiceClient},
    InvertedIndex,
};
use tarpc::{
    client, context,
    server::{self, Channel},
};
use tokio::sync::RwLock;

#[derive(Clone)]
struct HelloServer {
    inverted_index_lock: Arc<RwLock<InvertedIndex>>,
}

#[tarpc::server]
impl InvertedIndexService for HelloServer {
    async fn get(self, _: context::Context, term: String) -> Vec<String> {
        self.inverted_index_lock.read().await.get(&term)
    }

    async fn build(self, _: context::Context, paths: Vec<String>) {
        let mut inverted_index = self.inverted_index_lock.write().await;
        *inverted_index =
            InvertedIndex::build(paths.into_iter().map(|s| PathBuf::from(s)).collect(), 1);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    let server_data = HelloServer {
        inverted_index_lock: Arc::new(RwLock::new(InvertedIndex::new())),
    };

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

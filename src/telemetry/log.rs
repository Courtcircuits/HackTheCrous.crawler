use std::io;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use url::Url;

pub async fn init_logger(loki_endpoint: Option<String>){
    match loki_endpoint {
        Some(endpoint) => {
            loki_logger(endpoint).await.unwrap();
        }
        None => {
            default_logger();
        }
    }
}

fn default_logger() {
    tracing_subscriber::fmt::init();
}

async fn loki_logger(loki_endpoint: String) -> Result<(), tracing_loki::Error> {
    let std_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(io::stdout)
        .and_then(tracing_subscriber::EnvFilter::from_default_env())
        .boxed();


    let (layer, background_task) = tracing_loki::builder()
        .label("host", "mine")? //to change with relevant name
        .build_url(Url::parse(&loki_endpoint).unwrap())?;
    
    
    let layer = layer.and_then(tracing_subscriber::EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(layer)
        .with(std_layer)
        .try_init().unwrap();

    tokio::spawn(background_task);

    Ok(())
}

use std::{io, time::Duration};
use opentelemetry::trace::{Tracer, TracerProvider as _};


use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace::{RandomIdGenerator, Sampler, TracerProvider}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use url::Url;

use crate::cli;

pub async fn init_logger(loki_endpoint: Option<String>, subcommand: &cli::Command) {
    match loki_endpoint {
        Some(endpoint) => {
            loki_logger(endpoint, subcommand).await.unwrap();
        }
        None => {
            default_logger();
        }
    }
}

fn default_logger() {
    tracing_subscriber::fmt::init();
}

async fn loki_logger(
    loki_endpoint: String,
    subcommand: &cli::Command,
) -> Result<(), tracing_loki::Error> {
    let std_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(io::stdout)
        .and_then(tracing_subscriber::EnvFilter::from_default_env())
        .boxed();

    let (layer, background_task) = tracing_loki::builder()
        .label("job", subcommand.as_str())? //to change with relevant name
        .build_url(Url::parse(&loki_endpoint).unwrap())?;

    let layer = layer.and_then(tracing_subscriber::EnvFilter::from_default_env());

    let provider = init_tracer_provider();
    let tracer = provider.tracer("readme_example");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(layer)
        .with(telemetry)
        .with(std_layer)
        .try_init()
        .unwrap();

    tokio::spawn(background_task);

    Ok(())
}

fn init_tracer_provider() -> TracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://0.0.0.0:4317")
        .with_timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    let exporter = opentelemetry_stdout::SpanExporter::default();

    TracerProvider::builder()
        // Customize sampling strategy
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        // If export trace to AWS X-Ray, you can use XrayIdGenerator
        .with_id_generator(RandomIdGenerator::default())
        .with_batch_exporter(exporter, runtime::Tokio)
        .build()
}

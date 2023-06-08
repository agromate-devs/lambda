use std::time::Duration;
use aws_config::load_from_env;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3:: Client;
use lambda_http::{run, service_fn, Body, Error, Request, Response};

const BUCKET: &str = "plantsdb";    // Our bucket
const DB_FILE: &str = "usdadb_new.sqlite3"; // Our DB file

async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    let shared_config = load_from_env().await;
    let client = Client::new(&shared_config);

    let object = client // Get DB filestream
        .get_object()
        .bucket(BUCKET)
        .key(DB_FILE)
        .presigned(PresigningConfig::expires_in(Duration::new(20, 0)).unwrap())
        .await?;

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(object.uri().to_string().into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

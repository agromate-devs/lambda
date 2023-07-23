mod router;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response, http::Method};
use router::router;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    Ok(router(event).await.unwrap())
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

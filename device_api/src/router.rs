use lambda_http::{Request, Response, Body, Error};
use aws_sdk_dynamodb::{Client};
mod endpoints;
use endpoints::get_devices;
use self::endpoints::{not_implemented, add_devices, delete_devices};

pub async fn router(request: Request, client:  &Client) -> Result<Response<Body>, Error>{
    match request.method().as_str() {
        "GET" => get_devices(request, &client).await,
        "POST" => add_devices(request, &client).await,
        "DELETE" => delete_devices(request, &client).await,
        _ => not_implemented(),
    }
}

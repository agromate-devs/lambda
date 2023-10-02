mod models;
mod endpoints;

use lambda_http::http::Method;
use lambda_http::{Request, RequestExt, Response, Body};
use models::PostRequest;
use aws_config::load_from_env;
use aws_sdk_dynamodb::Client;
use helper::get_user_id;
use self::endpoints::{get_plant, add_plant};

pub async fn router(event: Request) -> Result<Response<Body>, Box<lambda_http::http::Error>>  {   // Router for our HTTP lambda
    let result: Result<Response<Body>, Box<lambda_http::http::Error>> = match event.method() {
        &Method::POST => {  // Add a new plant to DB
            let shared_config = load_from_env().await;
            let client = Client::new(&shared_config);
            let iot_dataplane_client = aws_sdk_iotdataplane::Client::new(&shared_config);
            let sns_client = aws_sdk_sns::Client::new(&shared_config);

            let body = event.body();
            let body_string = std::str::from_utf8(body).expect("invalid utf-8 sequence");
        
            let mut body_parsed: PostRequest = serde_json::from_str::<PostRequest>(body_string).unwrap();
            body_parsed.user_id = get_user_id(&event);  // Fill user_id with user_id from JWT token

            Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(add_plant(client, iot_dataplane_client, sns_client, body_parsed).await.unwrap().into())
            .map_err(Box::new)
        }
  
        &Method::GET => {
            let shared_config = load_from_env().await;
            let client = Client::new(&shared_config);
        
            let user_id = get_user_id(&event);
            let query_string =  event.query_string_parameters_ref().unwrap();
            let sensor_id = query_string.first("sensor_id").expect("Cannot parse sensor_id");

            Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(get_plant(client, &user_id, sensor_id).await.unwrap().into())
            .map_err(Box::new)
            
        }
        _ => {
            Response::builder()
            .status(200)
            .header("content-type", "text/plain")
            .body("Method not implemented".to_string().into())
            .map_err(Box::new)
        }
    };  
    result

}
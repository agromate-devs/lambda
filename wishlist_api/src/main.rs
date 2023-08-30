use std::collections::HashMap;
use aws_sdk_dynamodb::{operation::scan::ScanOutput, types::AttributeValue, Client};
use lambda_http::{http::Method, run, service_fn, Body, Error, Request, RequestExt , Response};
use response::{success_response, internal_server_error};
mod response;

const TABLE_NAME: &str = "wishlist"; // DynamoDB table name

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Plant {
    uuid: String, // ID of the plant(generated by client)
    name: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct WishListRequest {
    uid: String,  // Firebase User ID
    plant: Plant, // Plant name
    list_id: i8,  // Unique identifier of selected wishlist
}

#[derive(Clone, serde::Serialize)]
struct List {
    list_id: i8, // Unique identifier of list
    plants: Vec<Plant>,
    uuid: String,
}

async fn get_list(client: &Client, uid: &str) -> ScanOutput {
    let plants_in_list_raw = client
        .scan()
        .table_name(TABLE_NAME)
        .filter_expression("#uid = :id")
        .expression_attribute_names("#uid", "user_id")
        .expression_attribute_values(":id", AttributeValue::S(uid.to_string()))
        .send()
        .await
        .unwrap();

    plants_in_list_raw
}

async fn add_plant(client: &Client, details: WishListRequest) -> bool {
    let uid = AttributeValue::S(details.uid);
    let plant_name = AttributeValue::S(details.plant.name);
    let list_id = AttributeValue::N(details.list_id.to_string());
    let uuid = AttributeValue::S(details.plant.uuid);

    let request = client
        .put_item()
        .table_name(TABLE_NAME)
        .item("user_id", uid)
        .item("plant_name", plant_name)
        .item("list_id", list_id)
        .item("uuid", uuid);

    let response = request.send().await;

    match response {
        Ok(_response) => true,
        Err(error) => {
            println!("{:?}", error.raw_response());
            false
        }
    }
}

async fn delete_list(client: &Client, uid: &str, list_id: &str) -> bool {
    // TODO: Rewrite with bulk delete
    let lists = get_list(&client, uid).await;
    let selected_list: Vec<HashMap<String, AttributeValue>> = lists
        .items()
        .unwrap()
        .to_vec()
        .into_iter()
        .filter(|x| {
            x.get("list_id").unwrap().to_owned() == AttributeValue::N(list_id.to_string())
        })
        .collect();
    for plant in selected_list {
        client
            .delete_item()
            .table_name(TABLE_NAME)
            .key("uuid", plant.get("uuid").unwrap().clone())
            .key("user_id", AttributeValue::S(uid.to_string()))
            .send()
            .await
            .unwrap();
    }
    true
}

async fn delete_plant(client: &Client, plant_uuid: &str, uid: &str) -> bool {
    let request = client
        .delete_item()
        .table_name(TABLE_NAME)
        .key("uuid", AttributeValue::S(plant_uuid.to_string()))
        .key("user_id", AttributeValue::S(uid.to_string()))
        .send()
        .await;

    match request {
        Ok(_result) => true,
        Err(error) => {
            println!("{:?}", error.raw_response());
            false
        }
    }
}

fn hashmap_to_lists(plants: Vec<HashMap<String, AttributeValue>>) -> Vec<Plant> {
    let mut list_of_plants: Vec<Plant> = Vec::new();

    for plant in plants {
        list_of_plants.push(Plant {
            uuid: plant.get("uuid").unwrap().as_s().unwrap().to_string(),
            name: plant.get("plant_name").unwrap().as_s().unwrap().to_string(),
        })
    }
    list_of_plants
}
/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    match event.method() {
        &Method::POST => {
            let body = event.body();
            let body_string = std::str::from_utf8(body).expect("invalid utf-8 sequence");
        
            let body_parsed = serde_json::from_str::<WishListRequest>(body_string).unwrap();
        
            if add_plant(&client, body_parsed.clone()).await {
                Ok(success_response().unwrap())
            }else {
                Ok(internal_server_error("add_plant").unwrap())
            }
        }
        &Method::DELETE => {
            let uid = event
            .query_string_parameters_ref()
            .and_then(|params| params.first("uid"))
            .unwrap();
            let list_id = event
            .query_string_parameters_ref()
            .and_then(|params| params.first("list_id"))
            .unwrap();
            if delete_list(&client, uid, list_id).await {
                Ok(success_response().unwrap())
            }else {
                Ok(internal_server_error("delete_list").unwrap())
            }
        }
        &Method::PUT => {
            let uid = event
            .query_string_parameters_ref()
            .and_then(|params| params.first("uid"))
            .unwrap();
            let plant_uuid = event
            .query_string_parameters_ref()
            .and_then(|params| params.first("plant_uuid"))
            .unwrap();
            if delete_plant(&client, plant_uuid, uid).await {
                Ok(success_response().unwrap())
            }else {
                Ok(internal_server_error("delete_plant").unwrap())
            }

        }
        &Method::GET => {
            let uid = event
            .query_string_parameters_ref()
            .and_then(|params| params.first("uid"))
            .unwrap();
            // Get Lists and return they
            let lists = hashmap_to_lists(
                get_list(&client, uid)
                    .await
                    .items()
                    .unwrap()
                    .to_vec(),
            );

            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(serde_json::to_string(&lists).unwrap().into())
                .map_err(Box::new)?;
            Ok(resp)
        }
        _ => {
            let resp = Response::builder()
                .status(501)
                .header("content-type", "text/html")
                .body("Not implemented".into())
                .map_err(Box::new)?;
            Ok(resp)
        }
    }
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
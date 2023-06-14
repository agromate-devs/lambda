use std::collections::HashMap;

use aws_sdk_dynamodb::{operation::scan::ScanOutput, types::AttributeValue, Client};
use lambda_http::{http::Method, run, service_fn, Body, Error, Request, Response};

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

async fn get_list(client: &Client, details: WishListRequest) -> ScanOutput {
    let plants_in_list_raw = client
        .scan()
        .table_name(TABLE_NAME)
        .filter_expression("#uid = :id")
        .expression_attribute_names("#uid", "user_id")
        .expression_attribute_values(":id", AttributeValue::S(details.uid))
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

async fn delete_list(client: &Client, details: WishListRequest) -> bool {
    // TODO: Rewrite with bulk delete
    let lists = get_list(&client, details.clone()).await;
    let selected_list: Vec<HashMap<String, AttributeValue>> = lists
        .items()
        .unwrap()
        .to_vec()
        .into_iter()
        .filter(|x| {
            x.get("list_id").unwrap().to_owned() == AttributeValue::N(details.list_id.to_string())
        })
        .collect();
    for plant in selected_list {
        client
            .delete_item()
            .table_name(TABLE_NAME)
            .key("uuid", plant.get("uuid").unwrap().clone())
            .key("user_id", AttributeValue::S(details.uid.to_string()))
            .send()
            .await
            .unwrap();
    }
    true
}

async fn delete_plant(client: &Client, details: WishListRequest) -> bool {
    let request = client
        .delete_item()
        .table_name(TABLE_NAME)
        .key("uuid", AttributeValue::S(details.plant.uuid))
        .key("user_id", AttributeValue::S(details.uid))
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
    let body = event.body();
    let body_string = std::str::from_utf8(body).expect("invalid utf-8 sequence");

    let body_parsed = serde_json::from_str::<WishListRequest>(body_string).unwrap();

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let mut result = false;

    match event.method() {
        &Method::POST => {
            result = add_plant(&client, body_parsed.clone()).await;
        }
        &Method::DELETE => {
            result = delete_list(&client, body_parsed.clone()).await;
        }
        &Method::PUT => {
            result = delete_plant(&client, body_parsed.clone()).await;
        }
        _ => {}
    }

    if event.method() == &Method::GET {
        // Get Lists and return they
        let lists = hashmap_to_lists(get_list(&client, body_parsed)
        .await
        .items()
        .unwrap()
        .to_vec());

        let resp = Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(serde_json::to_string(&lists).unwrap().into())
            .map_err(Box::new)?;
        Ok(resp)
    } else {
        // Normal request
        // Return something that implements IntoResponse.
        // It will be serialized to the right response event automatically by the runtime
        let resp = Response::builder()
            .status(if result { 200 } else { 500 })
            .header("content-type", "text/html")
            .body(
                (if result {
                    "OK"
                } else {
                    "ERROR, malformed body or invalid method"
                })
                .into(),
            )
            .map_err(Box::new)?;
        Ok(resp)
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

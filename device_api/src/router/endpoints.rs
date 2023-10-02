use std::collections::HashMap;


use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_http::{Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use helper::get_user_id;

const DEVICE_TABLE_NAME: &str = "devices";

#[derive(Serialize, Deserialize)]
struct ResponseBody<'a> {
    error: bool,
    message: &'a str,
}

#[derive(Serialize, Deserialize)]
struct Item {
    user_id: String,
    device_id: String,
}

fn query_to_string(items: &[HashMap<String, AttributeValue>]) -> String {
    let mut items_parsed = Vec::new();
    for item_raw in items {
        for element in item_raw {
            let item = Item {
                user_id: element.0.to_string(),
                device_id: element.1.as_s().unwrap().to_string(),
            };
            items_parsed.push(item);
        }
    } 
    serde_json::to_string(&items_parsed).unwrap()
}

pub async fn get_devices(req: Request, client: &Client) -> Result<Response<Body>, Error> {
    let user_id = get_user_id(&req);

    let results = client
        .query()
        .table_name(DEVICE_TABLE_NAME)
        .key_condition_expression("#key = :value".to_string())
        .expression_attribute_names("#key", "user_id")
        .expression_attribute_values(":value", AttributeValue::S(String::from(user_id)))
        .send()
        .await
        .unwrap();
 
    let response = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(query_to_string(results.items().unwrap()).into())
        .map_err(Box::new)?;
    Ok(response)
}

pub async fn add_devices(req: Request, client: &Client) -> Result<Response<Body>, Error> {
    let user_id = get_user_id(&req);

    let board_uuid = req
        .query_string_parameters_ref()
        .and_then(|params| params.first("device_id"))
        .unwrap();

    let user_av = AttributeValue::S(user_id.to_string());
    let board_av = AttributeValue::S(board_uuid.to_string());

    let request = client
        .put_item()
        .table_name(DEVICE_TABLE_NAME)
        .item("user_id", user_av)
        .item("device_id", board_av);

    let resp = request.send().await;

    match resp {
        Ok(_e) => {
            let response_body = ResponseBody {
                error: false,
                message: "Device added successfully",
            };

            let response = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(serde_json::to_string(&response_body).unwrap().into())
                .map_err(Box::new)?;

            Ok(response)
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn delete_devices(req: Request, client: &Client) -> Result<Response<Body>, Error> {
    let user_id = get_user_id(&req);

    let board_uuid = req
        .query_string_parameters_ref()
        .and_then(|params| params.first("device_id"))
        .unwrap(); 
    
   let request = client
        .delete_item()
        .table_name(DEVICE_TABLE_NAME)
        .key("user_id", AttributeValue::S(user_id.into()))
        .key("device_id", AttributeValue::S(board_uuid.into()))
        .send()
        .await;
    
        match request {
        Ok(_out) => {
             let response_body = ResponseBody {
                error: false,
                message: "Device deleted successfully",
            };

            let response = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(serde_json::to_string(&response_body).unwrap().into())
                .map_err(Box::new)?;

            Ok(response)
        }
        Err(e) => {
            Err(e.into())
        },
    }
}

pub fn not_implemented() -> Result<Response<Body>, Error> {
    let response_body = ResponseBody {
        error: true,
        message: "API not implemented",
    };

    let response = Response::builder()
        .status(404)
        .header("content-type", "text/html")
        .body(serde_json::to_string(&response_body).unwrap().into())
        .map_err(Box::new)?;

    Ok(response)
}

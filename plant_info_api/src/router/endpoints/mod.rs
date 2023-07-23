use super::models::PostRequest;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

const TABLE_NAME: &str = "plants"; // DynamoDB table name

pub async fn get_plant(client: Client, uid: &str, sensor_id: &str) -> Result<String, String> {
    let results = client
        .query()
        .table_name(TABLE_NAME)
        .key_condition_expression("user_id = :id and sensor_id = :sid")
        .expression_attribute_values(":id", AttributeValue::S(uid.to_string()))
        .expression_attribute_values(":sid", AttributeValue::S(sensor_id.to_string()))
        .send()
        .await;

    match results {
        Ok(results) => {
            let plants: Vec<PostRequest> =
                results.items().unwrap().iter().map(|v| v.into()).collect();
            let plants_str: String = serde_json::to_string(&plants).unwrap();
            Ok(plants_str)
        }
        Err(err) => 
        {
            println!("{:?}", err.raw_response());            
            Err("Something went wrong while finding plant in DB".to_string())
        }
    }
}

pub async fn add_plant(client: Client, request: PostRequest) -> Result<String, String> {
    let request = client // Save all details of plant in dynamoDB so our ESP8266 can use it
        .put_item()
        .table_name(TABLE_NAME)
        .item("user_id", AttributeValue::S(request.user_id))
        .item("plant_name", AttributeValue::S(request.plant_name))
        .item("sensor_id", AttributeValue::S(request.sensor_id))
        // Start Temperature
        .item(
            "default_temperature",
            AttributeValue::N(request.default_temperature.to_string()),
        )
        .item(
            "temperature_limit",
            AttributeValue::N(request.temperature_limit.to_string()),
        )
        .item(
            "notify_wrong_temperaure",
            AttributeValue::Bool(request.notify_wrong_temperature),
        )
        // End Temperature
        // Start Humidity
        .item(
            "default_humidity",
            AttributeValue::N(request.default_humidity.to_string()),
        )
        .item(
            "humidity_limit",
            AttributeValue::N(request.humidity_limit.to_string()),
        )
        .item(
            "notify_wrong_humidity",
            AttributeValue::Bool(request.notify_wrong_humidity),
        )
        // End humidity
        // Start Precipitation
        .item(
            "default_precipitation",
            AttributeValue::N(request.default_precipitation.to_string()),
        )
        .item(
            "precipitation_limit",
            AttributeValue::N(request.precipitation_limit.to_string()),
        )
        .item(
            "notify_wrong_soil_humidity",
            AttributeValue::Bool(request.notify_wrong_soil_humidity),
        )
        // End precipitation
        // Start color
        .item(
            "default_light_color",
            AttributeValue::S(request.default_light_color),
        )
        .item(
            "light_time",
            AttributeValue::N(request.light_time.to_string()),
        )
        .item(
            "light_intensiy",
            AttributeValue::N(request.light_intensity.to_string()),
        )
        // End color
        .send()
        .await;

    match request {
        Ok(_) => Ok("Plant added correctly".to_string()),
        Err(_) => Err("Error during adding plant".to_string()),
    }
}

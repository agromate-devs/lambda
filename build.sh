#!/bin/bash

function give_iam_roles {
	arn=$1
	role=$(aws lambda get-function --function-name "$arn" | grep Role | awk -F '":' '{printf $2}' | tr "," "\0")
	aws iam attach-role-policy --role-name "$role" --policy-arn arn:aws:iam::aws:policy/AmazonDynamoDBFullAccess
	aws iam attach-role-policy --role-name "$role" --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
}

function deploy_device_api {
	cd device_api && cargo lambda build --release
	arn=$(cargo lambda deploy | awk -F'function arn:' '{print $2}' | tr -d '\n')
	give_iam_roles "$arn"
	echo $arn
}

function deploy_get_sensors_data {
	cd get_sensors_data && cargo lambda build --release && cargo lambda deploy
}

function deploy_mqtt_month_media_processor {
	cd mqtt_month_media_processor && cargo lambda build --release && cargo lambda deploy	
}

function deploy_plant_info_api {
	cd plant_info_api && cargo lambda build --release && cargo lambda deploy	
}

function deploy_wishlist_api {
	cd wishlist_api && cargo lambda build --release && cargo lambda deploy
}

function deploy_notification_sender {
	cd notification_sender && cargo lambda build --release && cargo lambda deploy
}

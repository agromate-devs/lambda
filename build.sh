function deploy_device_api {
	cd device_api && cargo lambda build --release
	echo $(cargo lambda deploy | awk -F'function arn:' '{print $2}' | tr -d '\n')
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


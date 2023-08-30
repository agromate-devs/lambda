#!/bin/bash
cargo lambda invoke --data-ascii "{ \"temperature\": 1, \"humidity\": 10, \"uuid\": \"test\", \"hour\": true, \"media_month\": false }"

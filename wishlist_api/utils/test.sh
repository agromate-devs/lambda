#!/bin/bash
cargo lambda invoke --data-file post_event.json
cargo lambda invoke --data-file post2_event.json
cargo lambda invoke --data-file post3_event.json
cargo lambda invoke --data-file get_event.json
cargo lambda invoke --data-file put_event.json
cargo lambda invoke --data-file delete_event.json

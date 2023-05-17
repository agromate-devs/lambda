rm -rf target/
cargo lambda build --release
cargo lambda deploy 

check-mold:
	readelf -p .comment target/debug/asapi

release:
	cargo build --release


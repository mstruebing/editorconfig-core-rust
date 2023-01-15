build:
	cargo build --release

run:
	cargo run -- Makefile ./core-test/README.md

test:
	cargo test

test-core: build
	cd core-test; \
		cmake ..
	cd core-test; \
		ctest \
		-E "^(octothorpe_in_value)$$" \
		--output-on-failure \

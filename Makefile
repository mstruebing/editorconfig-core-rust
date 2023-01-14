build: 
	cargo build --release

test:
	cargo test

test-core: build
	cd core-test; \
		cmake ..
	cd core-test; \
		ctest \
		-E "^(octothorpe_in_value)$$" \
		--output-on-failure \

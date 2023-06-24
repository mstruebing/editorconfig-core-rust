build:
	cargo build --release

run:
	cargo run -- Makefile ./core-test/README.md ./README.md

test:
	cargo test

test-core: build
	cd core-test; \
		cmake ..
	cd core-test; \
		ctest \
		-E "^(octothorpe_in_value)$$" \
		--output-on-failure \


stuff: build
	cd core-test && "/home/maex/projects/own/editorconfig-core-rust/target/release/editorconfig-core-rust" "-f" "meta.in" "/home/maex/projects/own/editorconfig-core-rust/core-test/meta/meta.c"

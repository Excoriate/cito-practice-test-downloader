.PHONY: run build clean

run:
	@echo "Enter the year for document download:"
	@read year; \
	cargo run -- --year $$year

build:
	cargo build --release

clean:
	cargo clean

.DEFAULT_GOAL := run

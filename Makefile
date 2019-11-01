BIN_DIR = $(CURDIR)/bin
DATA_DIR = $(CURDIR)/data
CARGO_TARGET_DIR ?= $(CURDIR)/target

clean:
	rm -rf $(BIN_DIR)
	rm -rf $(DATA_DIR)
	rm -rf $(CARGO_TARGET_DIR)

format:
	cargo fmt

protoc:
	./generate_proto.sh

build:
	cargo update -p protobuf --precise 2.8.0
	cargo build --release
	mkdir -p $(BIN_DIR)
	cp $(CARGO_TARGET_DIR)/release/bayard $(BIN_DIR)/

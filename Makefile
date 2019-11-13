BIN_DIR ?= $(CURDIR)/bin
VERSION ?=

ifeq ($(VERSION),)
  VERSION = $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard") | .version')
endif

clean:
	rm -rf $(BIN_DIR)
	cargo clean

format:
	cargo fmt

protoc:
	./generate_proto.sh

build:
	mkdir -p $(BIN_DIR)
	cargo update -p protobuf --precise 2.8.0
	cargo build --release
	cp -p ./target/release/bayard $(BIN_DIR)

build-docker:
	docker build -t bayardsearch/bayard:latest .
	docker tag bayardsearch/bayard:latest bayardsearch/bayard:$(VERSION)

push-docker:
	docker push bayardsearch/bayard:latest
	docker push bayardsearch/bayard:$(VERSION)

clean-docker:
	docker rmi -f $(shell docker images --filter "dangling=true" -q --no-trunc)

build-docs:
	mdbook build

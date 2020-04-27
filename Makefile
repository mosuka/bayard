BIN_DIR ?= $(CURDIR)/bin
DOCS_DIR ?= $(CURDIR)/docs
SERVER_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-server") | .version')
CLIENT_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-client") | .version')
REST_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-rest") | .version')
VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard") | .version')

.DEFAULT_GOAL := build

clean:
	rm -rf $(BIN_DIR)
	cargo clean

format:
	cargo fmt

build: format
	mkdir -p $(BIN_DIR)
	cargo build --release
	cp -p ./target/release/bayard $(BIN_DIR)
	cp -p ./target/release/bayard-rest $(BIN_DIR)

test:
	cargo test

tag:
	git tag v$(VERSION)
	git push origin v$(VERSION)

publish: format
ifeq ($(shell cargo show --json bayard-server | jq -r '.versions[].num' | grep $(SERVER_VERSION)),)
	(cd bayard-server && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-client | jq -r '.versions[].num' | grep $(CLIENT_VERSION)),)
	(cd bayard-client && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-rest | jq -r '.versions[].num' | grep $(REST_VERSION)),)
	(cd bayard-rest && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-client | jq -r '.versions[].num' | grep $(VERSION)),)
	(cd bayard && cargo package && cargo publish)
endif

docker-build:
	docker build -t bayardsearch/bayard:latest .
	docker tag bayardsearch/bayard:latest bayardsearch/bayard:$(VERSION)

docker-push:
	docker push bayardsearch/bayard:latest
	docker push bayardsearch/bayard:$(VERSION)

docker-clean:
	docker rmi -f $(shell docker images --filter "dangling=true" -q --no-trunc)

.PHONY: docs
docs:
	rm -rf $(DOCS_DIR)
	mdbook build

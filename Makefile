BIN_DIR ?= $(CURDIR)/bin
DOCS_DIR ?= $(CURDIR)/docs
BAYARD_COMMON_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-common") | .version')
BAYARD_SERVER_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-server") | .version')
BAYARD_CLIENT_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-client") | .version')
BAYARD_CLI_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-cli") | .version')
BAYARD_REST_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard-rest") | .version')
BAYARD_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard") | .version')

.DEFAULT_GOAL := build

clean:
	rm -rf $(BIN_DIR)
	cargo clean

format:
	cargo fmt

build:
	mkdir -p $(BIN_DIR)
	cargo build --release
	cp -p ./target/release/bayard $(BIN_DIR)
	cp -p ./target/release/bayard-cli $(BIN_DIR)
	cp -p ./target/release/bayard-rest $(BIN_DIR)

test:
	cargo test

tag:
	git tag v$(BAYARD_VERSION)
	git push origin v$(BAYARD_VERSION)

publish:
ifeq ($(shell cargo show --json bayard-common | jq -r '.versions[].num' | grep $(BAYARD_COMMON_VERSION)),)
	(cd bayard-common && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-server | jq -r '.versions[].num' | grep $(BAYARD_SERVER_VERSION)),)
	(cd bayard-server && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-client | jq -r '.versions[].num' | grep $(BAYARD_CLIENT_VERSION)),)
	(cd bayard-client && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-cli | jq -r '.versions[].num' | grep $(BAYARD_CLI_VERSION)),)
	(cd bayard-cli && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard-rest | jq -r '.versions[].num' | grep $(BAYARD_REST_VERSION)),)
	(cd bayard-rest && cargo package && cargo publish)
	sleep 10
endif
ifeq ($(shell cargo show --json bayard | jq -r '.versions[].num' | grep $(BAYARD_VERSION)),)
	(cd bayard && cargo package && cargo publish)
endif

docker-build:
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/bayardsearch/bayard/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_VERSION)),)
	docker build --tag=bayardsearch/bayard:latest --file=bayard.dockerfile --build-arg="BAYARD_VERSION=$(BAYARD_VERSION)" .
	docker tag bayardsearch/bayard:latest bayardsearch/bayard:$(BAYARD_VERSION)
endif
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/bayardsearch/bayard-rest/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_REST_VERSION)),)
	docker build --tag=bayardsearch/bayard-rest:latest --file=bayard-rest.dockerfile --build-arg="BAYARD_REST_VERSION=$(BAYARD_REST_VERSION)" .
	docker tag bayardsearch/bayard-rest:latest bayardsearch/bayard-rest:$(BAYARD_REST_VERSION)
endif
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/bayardsearch/bayard-cli/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_CLI_VERSION)),)
	docker build --tag=bayardsearch/bayard-cli:latest --file=bayard-cli.dockerfile --build-arg="BAYARD_CLI_VERSION=$(BAYARD_CLI_VERSION)" .
	docker tag bayardsearch/bayard-cli:latest bayardsearch/bayard-cli:$(BAYARD_CLI_VERSION)
endif

docker-push:
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/bayardsearch/bayard/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_VERSION)),)
	docker push bayardsearch/bayard:latest
	docker push bayardsearch/bayard:$(BAYARD_VERSION)
endif
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/bayardsearch/bayard-rest/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_REST_VERSION)),)
	docker push bayardsearch/bayard-rest:latest
	docker push bayardsearch/bayard-rest:$(BAYARD_REST_VERSION)
endif
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/bayardsearch/bayard-cli/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_CLI_VERSION)),)
	docker push bayardsearch/bayard-cli:latest
	docker push bayardsearch/bayard-cli:$(BAYARD_CLI_VERSION)
endif

docker-clean:
ifneq ($(shell docker ps -f 'status=exited' -q),)
	docker rm $(shell docker ps -f 'status=exited' -q)
endif
ifneq ($(shell docker images -f 'dangling=true' -q),)
	docker rmi -f $(shell docker images -f 'dangling=true' -q)
endif
ifneq ($(docker volume ls -f 'dangling=true' -q),)
	docker volume rm $(docker volume ls -f 'dangling=true' -q)
endif

.PHONY: docs
docs:
	rm -rf $(DOCS_DIR)
	mdbook build

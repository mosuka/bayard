BAYARD_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="bayard") | .version')

.DEFAULT_GOAL := build

clean:
	cargo clean

fmt:
	cargo fmt

build:
	cargo build --release

test:
	cargo test

tag:
	git tag v$(BAYARD_VERSION)
	git push origin v$(BAYARD_VERSION)

publish:
ifeq ($(shell curl -s -XGET https://crates.io/api/v1/crates/bayard | jq -r '.versions[].num' | grep $(BAYARD_VERSION)),)
	(cd bayard && cargo package --no-verify && cargo publish --no-verify)
	sleep 10
endif

docker-build:
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/mosuka/bayard/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_VERSION)),)
	docker build --tag=mosuka/bayard:latest --file=Dockerfile --build-arg="BAYARD_VERSION=$(BAYARD_VERSION)" .
	docker tag mosuka/bayard:latest mosuka/bayard:$(BAYARD_VERSION)
endif

docker-push:
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/mosuka/bayard/tags' | jq -r '."results"[]["name"]' | grep $(BAYARD_VERSION)),)
	docker push mosuka/bayard:latest
	docker push mosuka/bayard:$(BAYARD_VERSION)
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

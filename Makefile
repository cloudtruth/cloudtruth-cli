#
# Copyright (C) 2021 CloudTruth, Inc.
#

os_name := $(shell uname -s)
rustup_exists := $(shell which rustup)
rust_intended := 1.52.1
rust_installed := $(shell rustc -V | cut -d' ' -f2)
rust_bad_version := $(shell grep "RUST_VERSION:" .github/workflows/*.yml | grep -v "$(rust_intended)")

.DEFAULT = all
.PHONY = all
.PHONY += cargo
.PHONY += clean
.PHONY += help
.PHONY += image
.PHONY += integration
.PHONY += lint
.PHONY += precommit
.PHONY += precommit_test
.PHONY += prerequisites
.PHONY += shell
.PHONY += targets
.PHONY += test
.PHONY += version_check

all: precommit

### Commands for outside the container

image:
	docker build --build-arg user_uid=$(shell id -u) --build-arg user_gid=$(shell id -g) -t cloudtruth/cli . -f Dockerfile.dev

shell:
	docker run --rm --privileged=true \
		--group-add $(shell stat -c '%g' /var/run/docker.sock) \
		-v $(PWD):/home/dev/cli \
		-v $(HOME)/.cargo:/home/dev/.cargo \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-it cloudtruth/cli

### Commands for either outside or inside the container

# the client must be generated before building the Rust program that uses it
cargo: client
	cargo build

clean:
	rm -rf target/
	rm -rf client/

# client needs to re-generated when the openapi.yaml changes
client: openapi.yml
	openapi-generator generate \
		-i openapi.yml \
		-g rust \
		-o client \
		--additional-properties=packageName=cloudtruth-restapi,supportAsync=false
	python3 patch_client.py && cd client && cargo fmt && cargo build

lint:
	cargo fmt --all -- --check
	cargo clippy --all-features -- -D warnings
	shellcheck install.sh

precommit: version_check cargo precommit_test lint

prerequisites:
ifeq ($(rustup_exists),'')
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
endif
ifeq ($(rustup_exists),'')
	$(error "You need to add ~/.cargo/bin to your PATH")
endif
ifneq ($(rust_intended),$(rust_installed))
	rustup upgrade $(rust_intended)
else
	@echo "Already running rustc version: $(rust_intended)"
endif
ifeq ($(os_name),Darwin)
	brew install shellcheck libyaml;
else
	sudo apt-get install shellcheck python-yaml pkg-config;
endif
	python3 -m pip install --user --upgrade -r requirements.txt
	make -C tests $@

precommit_test:
	cargo test
	make -C tests $@

test: precommit_test
	make -C tests

integration: cargo
	make -C tests $@

version_check:
ifneq ($(rust_intended),$(rust_installed))
	$(error "Rustc compiler version expected $(rust_intended), got $(rust_installed)")
endif
ifneq ($(rust_bad_version),)
	$(error "GitHub action uses bad rustc version: $(rust_bad_version)")
endif
	@echo "Using rustc version: $(rust_intended)"

regen: cargo
	make -C tests $@

help: targets

targets:
	@echo ""
	@echo "cargo          - builds rust target"
	@echo "clean          - clean out build targets"
	@echo "client         - generate and build the cloudtruth-restapi library"
	@echo "image          - make the cloudtruth/cli docker container for development"
	@echo "integration    - runs the integration test against the live server"
	@echo "lint           - checks for formatting issues"
	@echo "precommit      - build rust targets, tests, and lints the files"
	@echo "precommit_test - runs the cargo tests"
	@echo "prerequisites  - install prerequisites"
	@echo "regen          - regenerate non-build artifacts"
	@echo "shell          - drop into the cloudtruth/cli docker container for development"
	@echo "test           - runs precommit tests, as well as integration tests"
	@echo "version_check  - checks rustc versions"
	@echo ""

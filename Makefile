#
# Copyright (C) 2021 CloudTruth, Inc.
#

os_name := $(shell uname -s)
rust_intended := 1.52.0
rust_installed := $(shell rustc -V | cut -d' ' -f2)
rust_bad_version := $(shell grep "RUST_VERSION:" .github/workflows/*.yml | grep -v "$(rust_intended)")

.PHONY: help image shell all cargo clean lint precommit precommit_test prerequisites test lint targets version_check

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

all: precommit

cargo:
	cargo build

clean:
	rm -rf target/

lint:
	cargo fmt --all -- --check
	cargo clippy --all-features -- -D warnings
	shellcheck install.sh

precommit: version_check cargo precommit_test lint

prerequisites:
ifneq ($(rust_intended),$(rust_installed))
	rustup upgrade $(rust_intended)
else
	@echo "Already running rustc version: $(rust_intended)"
endif
ifeq ($(os_name),Darwin)
	brew install shellcheck libyaml;
else
	sudo apt-get install shellcheck python-yaml;
endif
	make -C tests $@

precommit_test:
	cargo test

test: precommit_test
	make -C tests

version_check:
ifneq ($(rust_intended),$(rust_installed))
	$(error "Rustc compiler version expected $(rust_intended), got $(rust_installed)")
endif
ifneq ($(rust_bad_version),)
	$(error "GitHub action uses bad rustc version: $(rust_bad_version)")
endif
	@echo "Using rustc version: $(rust_intended)"

help: targets

targets:
	@echo ""
	@echo "cargo          - builds rust target"
	@echo "clean          - clean out build targets"
	@echo "image          - make the cloudtruth/cli docker container for development"
	@echo "lint           - checks for formatting issues"
	@echo "precommit      - build rust targets, tests, and lints the files"
	@echo "precommit_test - runs "
	@echo "prerequisites  - install prerequisites"
	@echo "shell          - drop into the cloudtruth/cli docker container for development"
	@echo "test           - runs precommit tests, as well as integration tests"
	@echo "version_check  - checks rustc versions"
	@echo ""

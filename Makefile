#
# Copyright (C) 2021 CloudTruth, Inc.
#

os_name := $(shell uname -s)

.PHONY: prerequisites cargo test lint targets precommit

all: precommit

prerequisites:
ifeq ($(os_name),Darwin)
	brew install shellcheck;
else
	sudo apt-get install shellcheck;
endif

lint:
	cargo fmt --all -- --check
	cargo clippy --all-features -- -D warnings
	shellcheck install.sh

cargo:
	cargo build

test:
	cargo test
	make -C tests

precommit: cargo test lint

targets:
	@echo ""
	@echo "precommit     - build rust targets, tests, and lints the files"
	@echo "cargo         - builds rust target"
	@echo "lint          - checks for formatting issues"
	@echo "test          - runs tests (no linting)"
	@echo "prerequisites - install prerequisites"
	@echo ""

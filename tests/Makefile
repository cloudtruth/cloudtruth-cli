
.DEFAULT = test
.PHONY = test
.PHONY += precommit
.PHONY += build

build:
	cargo build --tests

precommit: build

prerequisites:
	@echo "No prerequisites"

test:
	cargo test --tests

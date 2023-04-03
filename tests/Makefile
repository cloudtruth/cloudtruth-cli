
.DEFAULT = test
.PHONY = test
.PHONY += precommit
.PHONY += build

build:
	cargo build --tests

precommit: build

test:
	cargo test --tests

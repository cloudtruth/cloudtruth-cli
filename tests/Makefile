
.DEFAULT = integration-test
.PHONY = integration-test
.PHONY += precommit
.PHONY += build

build:
	cargo build --tests

precommit: build

integration-test:
	cargo nextest run -E 'kind(test)'

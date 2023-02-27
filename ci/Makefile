#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY = all
.PHONY += build
.PHONY += clean
.PHONY += cli-check
.PHONY += cli-gen
.PHONY += default
.PHONY += gen
.PHONY += help
.PHONY += prerequisites
.PHONY += precommit
.PHONY += targets
.PHONY += CLEANED # forces rebuild of generated files when directory is cleaned
.PHONY += RUST_SOURCES # forces build of Rust sources

actions_files := $(shell ls gh-actions/*.json 2> /dev/null || echo CLEANED)
docker_files := $(shell ls docker/Dockerfile* 2> /dev/null || echo CLEANED)
config_files := config.yaml templates/Dockerfile
ci_bin := ../target/release/ci

default: gh-actions docker

all: clean docker gh-actions cli-gen

clean:
	rm -rf $(shell ls gh-actions/*.json docker/Dockerfile*)
# rm ~/.actrc

prerequisites:
# python3 -m pip install --user --upgrade -r requirements.txt
# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

precommit: default cli-check
	@git ls-files -m -mo --exclude-standard |\
		grep -E '^docker/Dockerfile.*|^gh-actions/.*\.json$$' ||\
		exit 0 ;\
		echo 'Found unstaged changes to generated CI files. Please add them to the commit.' &&\
		exit 1


cli-check:
	python3 cli_walk.py check

cli-gen:
	python3 cli_walk.py generate

gh-actions: $(actions_files) $(config_files) $(ci_bin)
	$(ci_bin) --actions --verbose
	@touch gh-actions

docker: $(docker_files) $(config_files) $(ci_bin)
	$(ci_bin) --docker --verbose
	@touch docker

$(ci_bin) build: RUST_SOURCES #let Cargo detect changes
	cargo build --release

help: targets

targets:
	@echo ""
	@echo "cargo/build	  - build the executable "
	@echo "clean          - remove generated docker files"
	@echo "cli-check      - makes sure help from latest build is unchanged"
	@echo "cli-gen        - generates the cli help.txt"
	@echo "precommit      - makes sure cli parameters have not changed"
	@echo "prerequisites  - install prerequisites"
	@echo ""

# a force target for Rust builds. we can use Cargo to detect source file changes
RUST_SOURCES:
# needed when running "make all" so make doesn't complain about missing targets
$(docker_files) $(actions_files):
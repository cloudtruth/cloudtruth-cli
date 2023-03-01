#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY = all
.PHONY += build
.PHONY += clean
.PHONY += default
.PHONY += gen
.PHONY += help
.PHONY += prerequisites
.PHONY += precommit
.PHONY += docker
.PHONY += gha-matrices
.PHONY += help-text
.PHONY += targets
.PHONY += CLEANED # forces rebuild of generated files when directory is cleaned
.PHONY += RUST_SOURCES # forces build of Rust sources

actions_files := $(shell ls gha-matrices/*.json 2> /dev/null || echo CLEANED)
docker_files := $(shell ls docker/Dockerfile* 2> /dev/null || echo CLEANED)
config_files := config.yaml templates/Dockerfile
ci_bin := ../target/release/ci

default: gha-matrices docker

all: clean docker gha-matrices help-text

clean:
	rm -rf $(shell ls gha-matrices/*.json docker/Dockerfile*)
# rm ~/.actrc

prerequisites:
# python3 -m pip install --user --upgrade -r requirements.txt
# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

precommit: default
	@git ls-files -m -mo --exclude-standard |\
		grep -E '^docker/Dockerfile.*|^gha-matrices/.*\.json$$' ||\
		exit 0 ;\
		echo 'Found unstaged changes to generated CI files. Please add them to the commit.' &&\
		exit 1

gha-matrices: $(actions_files) $(config_files) $(ci_bin)
	$(ci_bin) generate gha-matrices --verbose
	@touch gha-matrices

docker: $(docker_files) $(config_files) $(ci_bin)
	$(ci_bin) generate docker --verbose
	@touch docker

help-text: build
	@rm -rf $(shell ls ../tests/help/*)
	$(ci_bin) generate help-text --verbose

$(ci_bin) build: RUST_SOURCES #let Cargo detect changes
	cargo build --release

help: targets

targets:
	@echo ""
	@echo "build		  - build the executable "
	@echo "clean          - remove generated docker files"
	@echo "docker         - generate CI Dockerfiles"
	@echo "gha-matrices   - generate GHA job matrices"
	@echo "help-text	  - generate help text test cases"
	@echo "precommit      - makes sure CI files are up-to-date"
	@echo ""

# a force target for Rust builds. we can use Cargo to detect source file changes
RUST_SOURCES:
# needed when running "make all" so make doesn't complain about missing targets
$(docker_files) $(actions_files):
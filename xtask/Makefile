#
# Copyright (C) 2021 CloudTruth, Inc.
#
.DEFAULT = ci
.PHONY = all
.PHONY += build
.PHONY += ci
.PHONY += clean
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

ci: clean gha-matrices docker

all: clean docker gha-matrices help-text

clean:
	rm -rf $(shell ls gha-matrices/*.json docker/Dockerfile*)
# rm ~/.actrc

prerequisites:
# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

precommit: gha-matrices docker
	@git ls-files -m -mo --exclude-standard |\
		grep -E '^docker/Dockerfile.*|^gha-matrices/.*\.json$$' ||\
		exit 0 ;\
		echo 'Found unstaged changes to generated CI files. Please add them to the commit.' &&\
		exit 1

gha-matrices: $(actions_files) $(config_files)
	cargo xtask generate-gha-matrices --verbose
	@touch gha-matrices

docker: $(docker_files) $(config_files)
	cargo xtask generate-docker --verbose
	@touch docker

help-text:
	@rm -rf $(shell ls ../examples/help-text/*)
	cargo xtask generate-help-text --verbose

build:
	cargo build --release

help: targets

targets:
	@echo ""
	@echo "build		  - build the executable "
	@echo "ci             - generate CI artifacts (equivalent to make docker and make gha-matrices)"
	@echo "clean          - remove generated docker files"
	@echo "docker         - generate CI Dockerfiles"
	@echo "gha-matrices   - generate GHA job matrices"
	@echo "help-text	  - generate help text test cases"
	@echo "precommit      - makes sure CI files are up-to-date"
	@echo ""

# needed when running "make all" so make doesn't complain about missing targets
$(docker_files) $(actions_files):
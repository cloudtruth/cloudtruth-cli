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
.PHONY += precommit
.PHONY += docker
.PHONY += gha-matrices
.PHONY += help-text
.PHONY += targets
.PHONY += CLEANED # forces rebuild of generated files when directory is cleaned
.PHONY += RUST_SOURCES # forces build of Rust sources

actions_files := $(shell ls gha-matrices/*.json 2> /dev/null || echo CLEANED)
docker_files := $(shell ls docker/Dockerfile* 2> /dev/null || echo CLEANED)
config_files := config.yaml ../xtask/templates/Dockerfile

ci: clean gha-matrices docker

clean:
	@echo "Cleaning generated CICD files.."
	@rm -f $(shell ls ../cicd/gha-matrices/*.json ../cicd/docker/Dockerfile* 2>/dev/null)
# rm ~/.actrc

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
	@echo "precommit      - makes sure CI files are up-to-date"
	@echo ""

# needed when running "make all" so make doesn't complain about missing targets
$(docker_files) $(actions_files):
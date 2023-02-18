#
# Copyright (C) 2021 CloudTruth, Inc.
#

.DEFAULT := ci
.PHONY = all
.PHONY += ci
.PHONY += clean
.PHONY += cli-check
.PHONY += cli-gen
.PHONY += help
.PHONY += prerequisites
.PHONY += precommit
.PHONY += targets

all: clean ci cli-gen

clean:
	find docker/ -name 'Dockerfile*' | xargs rm -f
	find actions-matrices/ -name '*.json' | xargs rm -f
	# rm ~/.actrc

prerequisites:
	python3 -m pip install --user --upgrade -r requirements.txt
	# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

ci:
	cargo run --release

precommit: cli-check ci

cli-check:
	python3 cli_walk.py check

cli-gen:
	python3 cli_walk.py generate

help: targets

targets:
	@echo ""
	@echo "clean          - remove generated docker files"
	@echo "cli-check      - makes sure help from latest build is unchanged"
	@echo "cli-gen        - generates the cli help.txt"
	@echo "precommit      - makes sure cli parameters have not changed"
	@echo "prerequisites  - install prerequisites"
	@echo "workflow-*     - build workflow files and docker files from templates"
	@echo ""

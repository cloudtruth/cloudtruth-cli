#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY = all
.PHONY += clean
.PHONY += cli-check
.PHONY += cli-gen
.PHONY += help
.PHONY += prerequisites
.PHONY += precommit
.PHONY += regen
.PHONY += targets
.PHONY += workflow-test-release

all: workflow-test-release

clean:
	find docker/ -name 'Dockerfile*' | xargs rm -f
	# rm ~/.actrc

prerequisites:
	python3 -m pip install --user --upgrade -r requirements.txt
	# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

lint:
	python3 -m pip install --user --upgrade flake8
	python3 -m flake8

workflow-test-release:
	python3 gen.py

precommit: cli-check

cli-check:
	python3 cli_walk.py check

cli-gen:
	python3 cli_walk.py generate

regen: workflow-test-release cli-gen

help: targets

targets:
	@echo ""
	@echo "clean          - remove generated docker files"
	@echo "cli-check      - makes sure help from latest build is unchanged"
	@echo "cli-gen        - generates the cli help.txt"
	@echo "precommit      - makes sure cli parameters have not changed"
	@echo "prerequisites  - install prerequisites"
	@echo "regen          - regenerates the help.txt, GH action files, and docker files"
	@echo "workflow-*     - build workflow files and docker files from templates"
	@echo ""

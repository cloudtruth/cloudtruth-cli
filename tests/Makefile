#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY = all
.PHONY += clean
.PHONY += cli-check
.PHONY += cli-gen
.PHONY += help
.PHONY += live_test
.PHONY += prerequisites
.PHONY += precommit_test
.PHONY += regen
.PHONY += targets
.PHONY += workflow-draft
.PHONY += workflow-prerelease

all: workflow-draft workflow-prerelease live_test

clean:
	rm -f docker/
	# rm ~/.actrc

prerequisites:
	python3 -m pip install --user --upgrade -r requirements.txt
	# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

workflow-draft:
	python3 gen.py -w draft

workflow-prerelease:
	python3 gen.py -w prerelease

live_test:
	python3 pytest/live_test.py

precommit_test: cli-check

cli-check:
	python3 cli_walk.py check

cli-gen:
	python3 cli_walk.py generate

regen: workflow-draft workflow-prerelease cli-gen

help: targets

targets:
	@echo ""
	@echo "clean          - remove generated docker files"
	@echo "cli-check      - makes sure help from latest build is unchanged"
	@echo "cli-gen        - generates the cli help.txt"
	@echo "live_test      - run tests against live service"
	@echo "precommit_test - makes sure cli parameters have not changed"
	@echo "prerequisites  - install prerequisites"
	@echo "regen          - regenerates the help.txt, GH action files, and docker files"
	@echo "workflow-*     - build workflow files and docker files from templates"
	@echo ""

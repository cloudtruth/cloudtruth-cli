#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY: clean help live_test prerequisites workflow-draft workflow-prerelease targets

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

help: targets

targets:
	@echo ""
	@echo "clean         - remove generated docker files"
	@echo "live_test     - run tests against live service"
	@echo "prerequisites - install prerequisites"
	@echo "workflow-*    - build workflow files and docker files from templates"
	@echo ""

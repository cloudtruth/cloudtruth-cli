#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY: clean prerequisites workflow-draft workflow-prerelease targets

all: workflow-draft workflow-prerelease

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

targets:
	@echo ""
	@echo "clean         - remove generated docker files"
	@echo "prerequisites - install prerequisites"
	@echo "workflow-*    - build workflow files and docker files from templates"
	@echo ""

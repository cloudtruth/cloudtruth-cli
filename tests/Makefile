#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY: clean prerequisites test workflow targets

all: workflow

clean:
	rm -f docker/Dockerfile.*
	# rm ~/.actrc

prerequisites:
	python3 -m pip install --user --upgrade -r requirements.txt
	# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

workflow:
	# generate GitHub Actions workflows
	python3 gen.py

targets:
	@echo ""
	@echo "workflow      - build test.yaml and docker files from templates"
	@echo "prerequisites - install prerequisites"
	@echo "clean         - remove generated docker files"
	@echo ""

#
# Copyright (C) 2021 CloudTruth, Inc.
#

# Ideally one could run the workflow locally.
# Sadly, testing with act needs a docker DinD container to work.

.PHONY: clean prerequisites test workflow

all: workflow test

clean:
	rm -f tests/Dockerfile.*
	# rm ~/.actrc

prerequisites:
	python3 -m pip install --user --upgrade -r requirements.txt
	@if [ -z `which pre-commit` ]; then \
	    echo "Add $HOME/.local/bin to your path (try source ~/.profile) and make prerequisites again."; exit 1; fi
	pre-commit install
	sudo apt-get install shellcheck
	# curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

test:
	shellcheck install.sh
	# act -s CT_API_KEY pull_request

workflow:
	# generate GitHub Actions workflows
	python3 gen.py

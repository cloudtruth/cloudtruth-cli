#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY = all
.PHONY += clean
.PHONY += help
.PHONY += integration
.PHONY += lint
.PHONY += prerequisites
.PHONY += precommit
.PHONY += targets

all: integration

clean:
	rm -f __pycache__/

prerequisites:
	@echo "No current requirements"

lint:
	python3 -m pip install --user --upgrade flake8
	python3 -m flake8

integration:
	python3 live_test.py

precommit: lint

help: targets

targets:
	@echo ""
	@echo "clean          - cleans up build/run artifacts"
	@echo "integration    - run integration tests against live service"
	@echo "precommit      - makes sure cli parameters have not changed"
	@echo "prerequisites  - install prerequisites"
	@echo ""

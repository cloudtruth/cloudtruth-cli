#
# Copyright (C) 2021 CloudTruth, Inc.
#

.PHONY = all
.PHONY += clean
.PHONY += help
.PHONY += integration
.PHONY += prerequisites
.PHONY += precommit
.PHONY += targets

all: integration

clean:
	rm -f __pycache__/

prerequisites:
	python3 -m pip install --user --upgrade -r requirements.txt
	
integration:
	python3 live_test.py

precommit:
	@echo "Nothing to do"

help: targets

targets:
	@echo ""
	@echo "clean          - cleans up build/run artifacts"
	@echo "integration    - run integration tests against live service"
	@echo "precommit      - makes sure cli parameters have not changed"
	@echo "prerequisites  - install prerequisites"
	@echo ""

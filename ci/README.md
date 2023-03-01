This crate contains scripts, configuration, and utilities for CI.

# Configuration 
The `config.yaml` file is used to configure all of the auto-generated artifacts in this crate.

# Generating CI Artifacts

Simply running `make` in this directory will build all of the artifacts in the `docker` and `gha-matrices` directories. You can also `make clean` or `make all` if you need to replace old artifacts. Note that `make all` will also run the help text generation, which may not be desired.

CI artifacts are built automatically on commit. The precommit script will warn you if it detects unstaged changes to CI artifacts. 

# Regenerating Help Text

We use [trycmd](https://docs.rs/trycmd/latest/trycmd/) to run basic CLI test cases. Each of the subcommand help texts have a test case in `../tests/help` written as a Markdown file.

If you change the help text of a command and would like to regenerate the test cases, you can do so by running `make help-text` from this directories Makefile or the top-level project Makefile.

# Directory Overview
* `templates` - Various templates used for auto-generation. These are [askama](https://docs.rs/askama/latest/askama/) templates with syntax heavily based on Jinja.
* `scripts` - POSIX scripts for various CI tasks, mostly used in the GHA workflows.
* `src` - The Rust crate source code. Contains all the auto-generation code.
* Directories for generated artifacts
  * `docker` - Auto-generated Dockerfiles based on `config.yaml` and `templates/Dockerfile`.
  * `gha-matrices` - Auto-generated JSON files used as build matrices in the `build-release.yml` and `test-release.yml` GitHub Actions workflows.

# Templates

The following templates control output of certain artifacts:

* `templates/Dockerfile` - Template for all Dockerfiles generated in `docker`.
* `templates/help-text.md` - Template for generated [trycmd](https://docs.rs/trycmd/latest/trycmd/) test cases.

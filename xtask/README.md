This crate follows the [cargo xtask](https://github.com/matklad/cargo-xtask) pattern for adding extension scripts to `cargo` for automation of development related tasks.

- [Using `cargo xtask`](#using-cargo-xtask)
- [Configuration](#configuration)
- [Commands](#commands)
  - [Generating CI Artifacts with `generate-docker` and `generate-gha-matrices`](#generating-ci-artifacts-with-generate-docker-and-generate-gha-matrices)
  - [Regenerating Help Text with `generate-help-text`](#regenerating-help-text-with-generate-help-text)
  - [Clean Up Integration Test Data With `cleanup`](#clean-up-integration-test-data-with-cleanup)
- [Directory Overview](#directory-overview)
  - [Templates](#templates)

# Using `cargo xtask`

You can invoke tasks via the following command:

```
cargo xtask <COMMAND>
```

Full description of commands is provided via `--help` option:

```
Usage: xtask [OPTIONS] <COMMAND>

Commands:
  cleanup                Bulk Data Cleanup
  generate-docker        Generate Dockerfiles
  generate-gha-matrices  Generate GitHub Actions job matrix data
  generate-help-text     Generate test cases for CLI help text
  help                   Print this message or the help of the given subcommand(s)

Options:
  -p, --pretty   pretty-print output
  -v, --verbose  verbose logging
  -h, --help     Print help
```

# Configuration  
The `config.yaml` file is used to configure all of the auto-generated artifacts in this crate.

# Commands

## Generating CI Artifacts with `generate-docker` and `generate-gha-matrices`

`generate-docker` and `generate-gha-matrices` output generated artifact files in the `cicd` directory of this repo. Refer to the README there for more information about those files.

CI artifacts are built automatically on commit. The precommit script will warn you if it detects unstaged changes to CI artifacts. 

## Regenerating Help Text with `generate-help-text`

We use [trycmd](https://docs.rs/trycmd/latest/trycmd/) to run basic CLI test cases. Each of the subcommand help texts have a test case in `../tests/help` written as a Markdown file.

If you change the help text of a command and would like to regenerate the test cases, you can do so by running `make help-text` from this directories Makefile or the top-level project Makefile. This cleans the help text examples directory and runs the `generate-help-text` command to repopulate it.

## Clean Up Integration Test Data With `cleanup`

The `cleanup` command will use the CloudTruth CLI to perform bulk data cleanup. This is useful when running integration tests because sometimes the data does not get fully cleaned at the end of test execution. The `cleanup` will use your current Cloudtruth CLI configuration to bulk delete resources that match the input substrings given via command-line arguments.

Example:
```sh
cargo xtask cleanup substr1 substr2 ...
```

# Directory Overview
* `templates` - Various templates used for auto-generation. These are [askama](https://docs.rs/askama/latest/askama/) templates with syntax heavily based on Jinja.
* `src` - The Rust crate source code. Contains all the code xtask commands.

## Templates

The following templates control output of certain artifacts:

* `templates/Dockerfile` - Template for all Dockerfiles generated in `docker`.
* `templates/help-text.md` - Template for generated [trycmd](https://docs.rs/trycmd/latest/trycmd/) test cases.

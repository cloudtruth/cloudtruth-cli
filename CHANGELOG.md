# 1.0.1 - 2021-07-26

* Pickup latest OpenAPI changes.
* Add HTTP Cookie header sent from the first response in subsequent requests.
* Remove generated client docs from repository.
* Restore GiHub CI testing.

# 1.0.0 - 2021-07-20

* Update to use REST API server (replaces GraphQL).
* Added `parameters unset` to support removing override for a specific environment.
* Integrations:
  * Simplified `explore` interface -- use FQN instead of NAME/PATH/\[TYPE\]
  * Adjusted information displayed for `list -v` -- show Status (and update time), remove Type
* Config:
  * `edit` displays edited file path.
  * Added `current` to display current configuration with sources.
* Added `--rename` option for `set` subcommand of `parameters`, `projects`, and `environments`. 
* Deprecated support for setting API key via `CT_API_KEY` (use `CLOUDTRUTH_API_KEY`).

# 0.5.4 - 2021-06-15

* Fix display issue for no parameters in default project.
* Allow configuration file to be edited, even when it is invalid YAML.
* Improved error message when attempting to delete a parameter that does not exist. 
* Allow request_timeout to be set in profile or environment variable. 
* Improved integration tests with timeouts and server_url settings.
* Add wrap()/unwrap() functions to secure secrets (tested but not used).
* Initial work for REST interface to CTAAS.

# 0.5.3 - 2021-05-27

* Parameters: added `--dynamic` flag to `parameter list --values` view FQN/JMES path values.
* Added `yaml` and `json` table formatting options to tables.
* Fixed issue with Windows pre-release GitHub action.
* Upgrade to Rust compiler 1.52.1.
* Added `make integration` target to facilitate easier integration testing.

# 0.5.2 - 2021-05-20

* Important changes:
  * Templates: must use `--secrets` or `-s` to display secret values.
  * Parameters: 
      * `list --values` display has new `Type` and `Secret` columns.
      * `export` filtering and sorts are now case-insensitive, secret parameters included in normal
        export with redacted value.
* Prefer to use `-y/--yes` instead of `--confirm` to avoid confirmation prompts.
* Integrations: new sub-command with `list` and `explore`.
* Parameters: added `--fqn` and `--jmes` options to set references to dynamic values.

# 0.5.1 - 2021-05-10

* Upload install.sh, install.ps1, and CHANGELOG.md as release assets.
* Enhance install.sh to be more tolerant of non-standard release tags.
* Improve error checking on release tag names.
* Improve pre-commit checks for Rust version, and CLI command changes.

# 0.5.0 - 2021-05-07

* Breaking changes:
  * Parameters: `set` must use `--secret` instead of `--secrets`.
  * Environments: `set` will error out if trying to change the parent (not just warn and succeed).
* Added installer scripts for Posix shells, and Windows PowerShell.
* Can use `CLOUDTRUTH_PROFILE` to set the configuration profile, in addition to `--profile` option.
* Projects: sub-command can also be accessed with `project`.
* Parameters: `list` uses consistent message for empty parameter list.
* Moved development information out of `README.md` and into `DEVELOPMENT.md`.
* Changed release process to test installer scripts.
* Changed CI process to run the integration tests (using `tests/pytest/live_test.py`).
* Updated to Rust version 1.52.0.

# 0.4.1 - 2021-04-30

* Bug fix: use the `--env <env>` argument.

# 0.4.0 - 2021-04-28

* Breaking changes:
  * Moved `templates getit` functionality to `parameters export`.
* Arguments `--project <proj>` and `--env <env>` can also be set using configuration profile, or 
  environment variable (e.g. `CLOUDTRUTH_PROJECT`, `CLOUDTRUTH_ENVIRONMENT`).
* Config: enhanced `list` with `--values`, `--format`, and `--secrets` options
* Environments: added `set` and `delete` 
* Parameters: added `export`
* Projects: added `set` and `delete`

# 0.3.0 - 2021-04-16

* Breaking changes: 
  * The `--profile` option no longer supports the short (`-p`) version to avoid confusion with 
    the newly supported `--project` option.
  * Parameters: `set` must provide `--value`, `--prompt`, or `--input` option to set the value.
* Added `--project` option to specify a non-default project
* Environments: enhanced `list` with `--values` and `--format` options
* Projects: added `list`
* Parameters: 
  * Added `delete`
  * Enhanced `set` to allow setting `description` and `secret` properties, along with options to 
    enter a value without showing on screen.
* Templates: enhanced `list` with `--values` and `--format` options
* Added RPM packages.

# 0.2.0 - 2021-04-06

* Use `CLOUDTRUTH_API_KEY` instead of `CT_API_KEY` to set API key in the environment.
* Templates: `getit` command to render implicit templates of all parameters for different
  environments
* Parameters: enhanced `list` command with `--values` flag and `--format` option to display 
  parameter information and values.
* Run: breaking changes to replace `--preserve` flag with `--inherit` option for more control.

# 0.1.3 - 2021-03-25

* Added `run` option to run a command from an environment with parameters injected from the CloudTruth CLI.

# 0.1.2 - 2021-02-24

* Added a new dependency on the _ca-certificates_ package for our Debian installer in order to handle SSL certificates
  issued by Amazon's CA.
* Improved error messages when operations fail due to access restrictions on the API key.
* Improved error messages for data validation issues.

# 0.1.1 - 2021-01-20

* Synchronized with the latest CloudTruth API
* Replaced separate parameter creation and update calls with new upsert operation to avoid race conditions in CloudTruth
  parameter store

# 0.1.0 - 2020-11-30

The first release of the CloudTruth CLI tool, being made available for:

* Linux (x86_64) (tarball & deb packages)
* Linux (ARMv6) (tarball & deb packages)
* Linux (AArch64) (tarball & deb packages)
* macOS (x86_64)
* Windows (x86_64)

This release includes the following functionality:

* Environments: list
* Parameters: list, get, and create/update
* Templates: list and evaluate
* Multi-profile configuration to support different API keys
* Shell file completion scripts

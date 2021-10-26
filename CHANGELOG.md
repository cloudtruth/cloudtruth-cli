# 1.0.12 - 2021-10-26

* Added parent project support:
  * Added `projects set --parent` argument to set/update a project parent.
  * Added `parent` name to the `project list --values` display.
  * Added `projects tree` to show dependencies between projects.
  * Added `parameters list --parents` to display parameters defined in a parent project.
  * Added `parameters list --children` to display parameters defined in a descendant project.
* Added username and role information to `config current`.
* Added validation checks/options after `config edit`.
* Added username to `template history` output.
* Added `users` commands to manage user accounts and invitations.
* Added `audit-logs` commands to view audit log information.
* Improved performance for several `templates` commands (e.g. `get`, `diff`, `validate`).

# 1.0.11 - 2021-10-13

* Added support for evaluated parameters:
  * Added `parameter set --evaluate` to enable parameters referencing templates or  parameters.
  * Added `parameter list --evaluated` to see both evaluated and unevaluated parameter values.
* Added `template difference` command to compare template versions and environments.
* Added `--rename` option to `environment tag set`.

# 1.0.10 - 2021-10-01

* Added `--as-of` to `templates get` for raw or evaluated template at specified date/time (or tag).
* Changes to `configuration profile set`:
  * Fixed issue with reporting wrong action.
  * Added ability to set a source-profile using `--source`.
* Improved error handling to provide more understandable information.
* Updated several help strings.

# 1.0.9 - 2021-09-24

* Enhanced `parameters set` to allow parameter creation without a value for any environment.
* Added `template validate` to check if an existing template still evaluates properly.
* Fixed `parameters environment --as-of` issue when using a tag name.
* Added `configuration profile` commands to avoid need to modify configuration file with editor.
  * Moved `configuration list` functionality into `configuration profile list`.
* Improved handling for many error responses to provide more understandable information.

# 1.0.8 - 2021-09-22

* Added `environments tag` command with the following sub-commands:
  * `list` - display tags for a specific environment.
  * `set` - create/update environment tag properties.
  * `delete` - delete environment tag.
* Added `--show-times` to `list` for `integrations`, `projects`, and `environments`.
* Changed parameter properties `dynamic` to `external`, and `static` to `internal`.
* Added Centos-8 RPM support.
* Improved debugging for failed tests.

# 1.0.7 - 2021-09-10

* Improved `templates` commands:
  * Added `history` command.
  * Better error feedback when `get` or `preview` fail due to external parameters.
  * Added `--as-of` argument to `preview`.
  * Added `--show-times` flag to `list`.
* Added `--as-of` argument to `parameter export`.
* Support additional date formats in `--as-of` arguments (e.g. `mm/dd/YYYY`, `mm-dd-YYYY`). 
* Improved integration tests to use environment variables.

# 1.0.6 - 2021-09-07

* Updated `parameters set` with type and rule arguments:
  * Added `--type <string|integer|bool>` for parameter type.
  * Added `--max`, `--min`, `--max-len`, `--min-len`, `--regex` to set the rules, and a `no` 
    version of each (e.g. `--no-min-len`) to delete the rules.
* Updated `parameters list` with type and rule information:
  * Added "Param Type" and "Rules" (count) to the `--values` output.
  * Added `--rules` view of existing rules.

# 1.0.5 - 2021-08-30

* Allow retrieving unevaluated template text using `template get --raw`.
* Add `login` and `logout` commands for API key management.
* Added `--show-times` flag to display created-at and modified-at values parameters `list` and 
  `environment` commands.
* Allow retrieving parameter information at the specified `--as-of` value for:
  * `parameters list`
  * `parameters get`
  * `parameters environment`
  * `run`
* Updated `parameter difference` command:
  * Added properties to include `created-at` and `modified-at`.
  * Change arguments to allow environment and/or time-based (using `--as-of`) differences.  
* Display complete parameter information with `--details` flag in `parameter get` command.

# 1.0.4 - 2021-08-13

* Added `template edit` for easier template modification.
* Added `environment tree` to display environment inheritance.
* Added `parameter environment` to display values of a single parameter in all environments.
* Require user to confirm a `parameter delete` using the `--yes` flag or a prompt. 
* Performance improvement for `parameter get`.
* Upgrade to Rust 1.54.

# 1.0.3 - 2021-08-06

* Added `parameters difference` command to compare parameters between two different environments.
* Performance improvements:
  * Reduce the number of instances where secrets are retrieved.
  * Get parameter details using retrieve (by identifier), instead of a filtered list.
  * Added REST profiling prints when `CLOUDTRUTH_REST_DEBUG` is true.
* Improved feedback for errors retrieving dynamic parameters.
* Use standard cookie handling.

# 1.0.2 - 2021-07-30

* Improved aliases (e.g. `list` now accepts `ls` or `l`).
* Templates: added `set`, `delete`, and `preview` commands.
* Fixed issue with multiple parameters with no value -- only some values shown in `parameters list`.
* Fixed a couple issues with `templates get`:
  * Improved feedback when there are no templates found.
  * No longer show secrets without `--secrets` specified.
* When fail to add a value for a new parameter, the parameter is now removed.

# 1.0.1 - 2021-07-28

* Pickup latest OpenAPI changes.
* Add HTTP Cookie header sent from the first response in subsequent requests.
* Remove generated client docs from repository.
* Restore GiHub CI testing.
* Removed RPM builds due to infrastructure failures.

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

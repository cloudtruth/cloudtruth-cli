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

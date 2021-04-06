# 0.2.0 - 2021-04-06

* Use `CLOUDTRUTH_API_KEY` instead of `CT_API_KEY` to set API key in the environment.
* Templates: `getit` command to render implicit dotenv shell template for all parameters
* Parameters: `list` has `--values` flag and `--format` option to display parameter information and 
  values.
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

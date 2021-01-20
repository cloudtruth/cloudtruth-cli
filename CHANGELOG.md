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

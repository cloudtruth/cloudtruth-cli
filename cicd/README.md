This directory contains scripts, configuration, and generated files for CICD

# Generating CI Artifacts

Simply running `make` in this directory will build all of the artifacts in the `docker` and `gha-matrices` directories.

CI artifacts are built automatically on commit. The precommit script will warn you if it detects unstaged changes to CI artifacts.

# Configuration 
The `config.yaml` file is used to configure all of the auto-generated artifacts in this crate.

# Using `cargo xtask`
The `xtask` crate of this Cargo workspace is used to invoke tasks related to these files. See the README for the `xtask` directory for more information.

# Directory Overview
* `scripts` - POSIX scripts for various CI tasks, mostly used in the GHA workflows.
* `docker` - Auto-generated Dockerfiles based on `config.yaml` and `templates/Dockerfile`.
* `gha-matrices` - Auto-generated JSON files used as build matrices in the `build-release.yml` and `test-release.yml` GitHub Actions workflows.



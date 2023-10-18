CloudTruth CLI Development
==========================

Getting Started
--------

### Installing The Development Environment

The quickest way to install everything you need is to use the [Makefile](Makefile):

`make prerequisites`

This will:
* Setup the Rust development toolchain
  * install [rustup](https://rustup.rs/), a tool for managing Rust toolchain versions
  * use rustup to install the Rust toolchain version configured in the [rust-toolchain.toml](rust-toolchain.toml) file
  * install additional Rust toolchain components for formatting, linting, and docs
* Install Rust crates that are used during development
  * cargo-binstall, a utility for quickly installing Rust crates from pre-built binaries rather than compiling from source
  * taplo-cli, a TOML file linter and formatter
  * cargo-nextest, a test runner
* Install Shellcheck, for checking shell scripts
* Install Python modules used for developing Python scripts
  * black, an autoformatter for Python
  * yamllint, for linting YAML files
  * ruff, for linting Python scripts

### Configuring The CLI

To use the CLI, you'll need to configure a profile with a server URL and API key. Details for how to do this can be found in the [README](README.md)

Building
--------

The CloudTruth CLI is written in Rust, using the Rust 2021 edition. The target rustc version is configured in the [rust-toolchain.toml](rust-toolchain.toml) file. To build the application with debug info, check out the source code and then run:

`cargo build`

To build with release optimizations, run:

`cargo build --release`


Developing
----------

This project uses [rusty-hook](https://github.com/swellaby/rusty-hook) to help ensure commits pass 
tests and conform to the project's code style. These checks are run automatically as a git pre-commit hook to help cut down on "fix formatting" or "address linter" commits.

You do not need to explicitly write your own git pre-commit hook &mdash; rusty-hook will take care of that for you the first time you build the project. The pre-commit checks use `shellcheck` to check shell scripts.  You can run `make prerequisites` to install `shellcheck`.

### Formatting

The project uses `rustfmt` to maintain consistency with the prevailing formatting standard. It can be invoked with:

`cargo fmt`

### Linting

The project uses `clippy` to catch potentially problematic code patterns and ensure a consistent approach to solving problems.

It can be invoked with either of the following:

```
cargo clippy
make lint
```

To automatically fix linter issues you can run:

```
cargo clippy --fix
```

Testing
-------

The project has unit and integration tests. More details about each test variety is given below.

### Unit Tests

The unit tests can be run with:

```
cargo nextest run -E 'kind(lib) + kind(bin)'
```

for convenience you can use the shorthand alias `cargo xtest` (defined in [.cargo/config.toml](.cargo/config.toml))

```
cargo xtest
```

### Integration Tests

The integration tests run against the CloudTruth service, and verify a wide range of CLI functionality. To run the tests, you'll need an API key with admin access.

The tests live in the `tests` directory and can be run using cargo-nextest:

```
cargo nextest run -E 'kind(test)'
```

Nextest has many options for filtering tests and configuring output. See the [Nextest docs](https://nexte.st/index.html) for more information.

#### Cleaning Up Test Data

The integration tests generate a lot of test data. It makes a best effort to clean up after itself, but some data might leak after the test finishes. To make deleting test data faster, you can use the cleanup script in the `xtask` directory.

To run from command line:

```
cargo xtask cleanup [substrings ...]
```

The script will search for all test data whose names contain one of the given substrings and then ask you if you'd like to delete them.

Debugging
---------

This project makes use of the semi-standard Rust [log crate](https://crates.io/crates/log) to provide runtime logging. In order to see the log, you can set the `RUST_LOG` environment value to a [standard log level value](https://docs.rs/log/0.4.14/log/enum.Level.html). Notably, our HTTP client library will emit a lot of helpful information about the request and response cycle at the _trace_ level.

You can also use the `CLOUDTRUTH_REST_DEBUG` environment variable to configuring logging of API requests. Setting the value to `true` will log all requests, and setting it to a specific string will filter logs to only show that particular request ID.

Releases
-------------------

Generation of release artifacts is done using a GitHub Actions workflow and in many cases cannot be 
done in a local development environment.  To test changes to the artifact output, you can follow 
this workflow:

1. Make your code changes and push to a branch.
2. Create a tag for your branch and push following SemVer rules, for example _0.1.3-pre_.
3. This creates a draft release and you can check the results in the Actions tab.
   a. The GitHub actions install on several platforms, and verify the `cloudtruth` command can
      fetch a small set of data using the ci@cloudtruth.com account.
4. You can delete the draft release and the artifacts after you are done, then submit a pull request
   to get your changes into the _main_ branch.

Debugging
-----------------------------------------------------------

### Running a multi-command scenario with debugging in VS Code
With vscode-lldb you can debug Rust programs in VS Code. However, the launch configuration only has support for running a single executable file. For debugging a complex multi-command scenario, you need to invoke the debugger via external commands.

The following is an example script that you can use as a template. You'll need the vscode-lldb extension installed for this to work.

```sh
#!/usr/bin/env sh
set -e

### Path to local cloudtruth executable with debugging symbols.
### Make sure you're not building with the --release (-r) option for this.
CLOUDTRUTH_BIN="$(git rev-parse --show-toplevel)/target/debug/cloudtruth"

### alias the cloudtruth command to ignore PATH and ensure we always go to the local debug executable
# shellcheck disable=SC2139
alias cloudtruth="$CLOUDTRUTH_BIN"

### Use this function to launch lldb in vs code
### Example:
### debugcloudtruth projects list
debugcloudtruth () {
    args='['
    for arg in "$@"; do
        args="$args'$arg', "
    done
    args="$args]"
    launch_config="{type: 'lldb', request: 'launch', name: 'CloudTruth', sourceLanguages: ['rust'], program: '\${fileWorkspaceFolder}/target/debug/cloudtruth', args: $args}"
    debug_url='vscode://vadimcn.vscode-lldb/launch/config'
    code --wait --reuse-window --verbose --open-url "$debug_url?$launch_config"
}


# use `cloudtruth` to run commands without debugging
cloudtruth --version

# use `debugcloudtruth` to attach the debugger
debugcloudtruth --version
```

Windows builds on Linux/MacOS with MingGW
------------------------------------------

If you'd like to verify that your code builds on Windows from a local Linux or MacOS machine, you can install and configure the mingw-w64 runtime as a target for the compiler.

First you need to install mingw-w64. Use your OS package manager (`apt-get` for Debian/Ubuntu, `brew` for MacOS)

```
brew install mingw-w64
sudo apt-get install mingw-w64
```

Then you will need to register mingw-w64 as a custom target. Create or update your `~/.cargo/config` so that it contains:
```
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-gcc-ar"
```

You will also need to add the target to your rust toolchain otherwise your builds will not have `rust-std`
```
rustup target add x86_64-pc-windows-gnu
```

Now you can use cargo to build a Windows exe with your custom target
```
cargo build --release --target=x86_64-pc-windows-gnu
```





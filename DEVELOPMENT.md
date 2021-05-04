CloudTruth CLI Development
==========================

The CloudTruth CLI is an open source project designed for interacting with the 
[CloudTruth configuration management service](https://cloudtruth.com).

You are free to build it yourself, fork it as you see fit, or propose changes via the GitHub Pull 
Request mechanism.

Building
--------

The CloudTruth CLI tool is written in Rust, using the Rust 2018 edition.
While we don't actively pick the latest Rust features to use, we also haven't guaranteed it will 
build with older versions of Rust either.
If you have difficulties building, please check to see if a newer Rust will work.
Our CI configuration indicates which version we're using at the moment.

To build the application, check out the source code and then run:

`cargo build --release`

Developing
----------

This project uses [rusty-hook](https://github.com/swellaby/rusty-hook) to help ensure commits pass 
tests and conform to the project's code style.
These checks are run automatically as a git pre-commit hook to help cut down on "fix formatting" or 
"address linter" commits.
You do not need to explicitly write your own git pre-commit hook &mdash; rusty-hook will take care 
of that for you the first time you build the project.
The pre-commit checks use `shellcheck` to check the `install.sh`.  You can run `make prerequisites` 
to install `shellcheck`.

### Formatting

The project uses `rustfmt` to maintain consistency with the prevailing formatting standard.
It can be invoked with:

`cargo fmt`

### Linting

The project uses `clippy` to catch potentially problematic code patterns and ensure a consistent 
approach to solving problems.
It can be invoked with either of the following:

```
cargo clippy
make lint
```

Testing
-------

The project has unit and integration tests. More details about each test variety is given below.

### Unit Tests

The unit tests are the preferred place to check that code is functioning properly. It should be 
easiest to do negative testing in the unit test framework. We are making efforts to increase the
ability to unit test blocks of code.

The unit tests can be run with either:

```
cargo test
make test
```

If you run into test failures, try this remedy:

- unset `CLOUDTRUTH_API_KEY`
- `cargo test`

### Integration Tests

The integration tests run against the CloudTruth service, and verify a wide range of CLI 
functionality. The integration test requires an API key with write access.  Here's a quick sampling
of the functions validated by the integration test:
* Project create/update/delete
* Environment create/update/delete
* Parameter set/get/update/export
* Run inheritance validation
* Argument project/environment resolution

The integration test lives in `tests/pytest` and can be run using one of the following:
```
python3 tests/pytest/live_test.py
make test
```

The `live_test.py` has many options for filtering tests, debugging, and displaying output.

Debugging
---------

This project makes use of the semi-standard Rust [log crate](https://crates.io/crates/log) to 
provide runtime logging.
In order to see the log, you can set the `RUST_LOG` environment value to a 
[standard log level value](https://docs.rs/log/0.4.14/log/enum.Level.html).
Notably, our HTTP client library will emit a lot of helpful information about the request and 
response cycle at the _trace_ level.

Artifact Generation
-------------------

Generation of build artifacts is done using a GitHub Actions workflow and in many cases cannot be 
done in a local development environment.  To test changes to the artifact output, you can follow 
this workflow:

1. Make your code changes and push to a branch.
2. Create a tag for your branch and push following SemVer rules, for example _0.1.3-pre_.
3. This creates a draft release and you can check the results in the Actions tab.
   a. The GitHub actions install on several platforms, and verify the `cloudtruth` command can
      fetch a small set of data using the ci@cloudtruth.com account.
4. You can delete the draft release and the artifacts after you are done, then submit a pull request
   to get your changes into the _master_ branch.

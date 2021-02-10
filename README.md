CloudTruth CLI
==============

The CloudTruth CLI tool is used for interacting with the [CloudTruth configuration management service](https://cloudtruth.com).
In order to use this utility you must have an active CloudTruth account and generate a personal access token for authenticating with the API.

Prerequisites
-------------

The CloudTruth service is secured using SSL certificates issued by Amazon's CA service.
Amazon's CA is trusted by recent operating systems and browsers out of the box, but may require installation of an updated CA certificate package for older operating system releases.
E.g., Debian-based systems may require the installation of the _ca-certificates_ package.
If you install our Debian package, the necessary certificates package will be installed automatically.
For other systems, you may have to see your operating system vendor or distribution provides if you see any SSL-related errors when running the `cloudtruth` application.

Configuration
-------------

The CloudTruth CLI tool has several ways of configuring it, allowing you to pick whichever method is most suitable for your environment.
You can configure the application using:

* A configuration file
* Environment variables
* Command-line arguments


### File-based Configuration

The CloudTruth CLI utility stores its configuration in the YAML format.
The configuration file must be named _cli.yml_ and must reside in the standard application configuration location for your platform:

* Linux -> $XDG_CONFIG_HOME/cloudtruth/cli.yml
* macOS -> $HOME/Library/Application Support/com.cloudtruth.CloudTruth-CLI/cli.yml
* Windows -> %AppData%\CloudTruth\CloudTruth CLI\config\cli.yml

You can run `cloudtruth config edit` to initialize and open the configuration in your default editor.

The available configuration options are:

```yaml
--- 
profiles:
  default:
    api_key: <Your personal access token>

  another_profile:
    source_profile: default
    api_key: <Another personal access token>
```

Note that you can have multiple named profiles in your configuration, allowing you to maintain multiple sets of configuration fields in the configuration file.
Values can be inherited from one profile to another by way of the `source_profile` configuration field.
Profiles without an explicit `source_profile` configuration implicitly inherit from the _default_ profile.
You may choose which profile to use by passing the `--profile` to the CloudTruth CLI binary:

`cloudtruth --profile another-profile <subcommand>`

If the `--profile` argument is not supplied, the profile named _default_ will be used.

### Environment-based Configuration

The CloudTruth CLI utility maps all environment variables with the `CT_` followed by a configuration name to the same settings as would be available in the configuration file format.
If a configuration file is found, the configuration values from the environment will be merged with and take precedence over any values from the configuration file.

The available configuration options are:

`$CT_API_KEY` -> Your personal access token


### Argument-base Configuration

The CloudTruth CLI utility is able to be configured by supplying command-line arguments for the necessary settings.
As with the other configuration options available to you, care should be taken to ensure secret values are not exposed to other users on your system.
You may want to prefix commands with at space " " in order to avoid having secret values stored in your shell history.

The available configurations options can be displayed by running the tool with the `--help` or `-h` options:

`cloudtruth --help`, `cloudtruth -h`, or `cloudtruth help`.


Running
-------

Once you have the application configured with your CloudTruth API key, you can interact with your CloudTruth data subject to the access restrictions applied to the API key.
The CloudTruth CLI application uses a subcommand structure to scope available actions as appropriate for a given resource. You can see the full list of commands by running:

`cloudtruth help`

To make command discovery easier you can also get auto-completions for most popular shells by running:

`cloudtruth completions <SHELL>`, where "SHELL" is the name of your shell.

To see the full list of supported shells, you can run:

`cloudtruth completions --help`

All subcommands support a `--help` option to show you how the command should be invoked.

### Switching Active Configuration Profile

CloudTruth CLI profiles are a way of organizing the application's configuration data into multiple named groups.
Profiles, in this sense, are unrelated to configuration values in your CloudTruth account.
They simply allow you to configure the CloudTruth CLI for multiple organizations or multiple API keys with different access restrictions.
By default, the profile named _default_ will be used, but you can select the active profile with the `--profile` flag:

`cloudtruth --profile my-profile parameters get my_param`

### Switching Active CloudTruth Environment

By default, all commands will run against the _default_ CloudTruth environment.
To change the target environment, you can supply the global `--env` flag:

`cloudtruth --env production parameters get my_param`


Development
-----------

The CloudTruth CLI tool is open source.
You are free to build it yourself, fork it as you see fit, or propose changes via the GitHub Pull Request mechanism.

### Building

The CloudTruth CLI tool is written in Rust, using the Rust 2018 edition.
While we don't actively pick the latest Rust features to use, we also haven't guaranteed it will build with older versions of Rust either.
If you have difficulties building, please check to see if a newer Rust will work.
Our CI configuration indicates which version we're using at the moment.

To build the application, check out the source code and then run:

`cargo build --release`

### Developing

This project uses [rusty-hook](https://github.com/swellaby/rusty-hook) to help ensure commits pass tests and conform to the project's code style.
These checks are run automatically as a git pre-commit hook to help cut down on "fix formatting" or "address linter" commits.
You do not need to explicitly write your own git pre-commit hook &mdash; rusty-hook will take care of that for you the first time you build the project.

#### Tests

The tests can be run via `cargo`:

`cargo test`

#### Formatting

The project uses rustfmt to maintain consistency with the prevailing formatting standard.
It can be invoked with:

`cargo fmt`

#### Linting

The project uses Clippy to catch potentially problematic code patterns and ensure a consistent approach to solving problems.
It can be invoked with:

`cargo clippy`

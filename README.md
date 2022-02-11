CloudTruth CLI
==============

The CloudTruth CLI tool is used for interacting with the [CloudTruth configuration management service](https://cloudtruth.com).
In order to use this utility you must have an active CloudTruth account and generate a personal access token for authenticating with the API.

You can create a free account here [[Try Free](https://app.cloudtruth.io/signup)] and then create an API token [here](https://app.cloudtruth.io/organization/api)

Install
-------
### Homebrew (MacOS or Linux)

Use homebrew

```shell
brew install cloudtruth/formulae/cloudtruth-cli
```

See also [homebrew-formulae](https://github.com/cloudtruth/homebrew-formulae/blob/main/README.md)

### PowerShell (Windows)
<!-- (I did not find cloudtruth available via Windows package manager https://community.chocolatey.org/packages?q=cloudtruth) -->

See [install.ps1](install.ps1)

Run the following command to verify successfull installation and see the version.
```shell
cloudtruth --version
```


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
    description: Profile for different user
    # project: <something other than default?>
    # environment: <something other than default>
```

Note that you can have multiple named profiles in your configuration, allowing you to maintain multiple sets of configuration fields in the configuration file.
Values can be inherited from one profile to another by way of the `source_profile` configuration field.
Profiles without an explicit `source_profile` configuration implicitly inherit from the _default_ profile.
You may choose which profile to use by passing the `--profile` to the CloudTruth CLI binary:

`cloudtruth --profile another-profile <subcommand>`

If the `--profile` argument is not supplied, the profile named _default_ will be used.

### Environment-based Configuration

The CloudTruth CLI utility maps all environment variables with the `CLOUDTRUTH_` followed by a configuration name to the same settings as would be available in the configuration file format.
If a configuration file is found, the configuration values from the environment will be merged with and take precedence over any values from the configuration file.

The available configuration options are:

* `$CLOUDTRUTH_API_KEY` -> Your personal access token
* `$CLOUDTRUTH_PROFILE` -> Your profile (which can contain API key, project, and environment)
* `$CLOUDTRUTH_PROJECT` -> Your "default" CloudTruth project
* `$CLOUDTRUTH_ENVIRONMENT` -> Your "default" CloudTruth environment


### Argument-based Configuration

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


### CLI-API TLS

The CloudTruth service is secured using SSL certificates issued by Amazon's CA service.
Amazon's CA is trusted by recent operating systems and browsers out of the box, but may require installation of an updated CA certificate package for older operating system releases.

For example, Debian-based systems may require the installation of the _ca-certificates_ package.
If you install our Debian package, the necessary certificates package will be installed automatically.
For other systems, you may have to see your operating system vendor or distribution provides if you see any SSL-related errors when running the `cloudtruth` application.

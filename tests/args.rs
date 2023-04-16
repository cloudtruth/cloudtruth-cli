use cloudtruth_config::{CT_PROJECT, CT_REQ_TIMEOUT, CT_SERVER_URL};
use const_format::formatcp;
use integration_test_harness::prelude::*;

#[integration_test]
#[ignore]
fn test_arg_priority() {
    //TODO: original python test works around limitations of profiles
    //we should improve profile loading from custom file paths to make
    //this test robust
}

#[integration_test]
fn test_args_missing_subcommands() {
    let proj = Project::with_prefix("missing-subarg").create();
    let subcmds = [
        ("actions", vec!["action", "act", "ac"]),
        (
            "actions imports",
            vec![
                "action import",
                "action imp",
                "act import",
                "act im",
                "act i",
                "ac imports",
            ],
        ),
        (
            "actions pushes",
            vec!["action push", "action pu", "act pushes", "act pu", "act p"],
        ),
        ("audit-logs", vec!["audit", "aud", "au", "log", "logs"]),
        ("configuration", vec!["config", "conf", "con", "co", "c"]),
        (
            "configuration profiles",
            vec!["config profile", "conf prof", "c p"],
        ),
        ("environments", vec!["environment", "envs", "env", "e"]),
        ("environments tag", vec!["environment tag", "env ta"]),
        ("generate", vec!["gen", "ge"]),
        ("import", vec!["imp", "im"]),
        ("integrations", vec!["integration", "integrate", "int"]),
        (
            "parameters",
            vec!["parameter", "params", "param", "par", "pa", "p"],
        ),
        ("parameter-types", vec!["param-type", "types", "type", "ty"]),
        ("projects", vec!["project", "proj"]),
        ("run", vec!["ru", "r"]),
        ("templates", vec!["template", "te", "t"]),
        ("users", vec!["user", "us", "u"]),
        ("users invitations", vec!["user invitation", "us in", "u i"]),
        ("versions", vec!["version", "vers", "ver", "ve", "v"]),
    ];
    for (subcmd, aliases) in subcmds {
        for alias in core::iter::once(subcmd).chain(aliases) {
            cloudtruth!("{alias}")
                .env(CT_PROJECT, proj.name())
                .assert()
                .success()
                .stderr(contains!("WARN: No '{subcmd}' sub-command executed."));
        }
    }
}

#[integration_test]
fn test_arg_resolution() {
    // generate project and environment names but do not create them yet
    let proj = Project::with_prefix("unknown-proj");
    let env = Environment::with_prefix("env-unknown");
    let checked_commands = [
        "param ls -v",
        "templates ls -v",
        formatcp!("run -i none -c {}", DISPLAY_ENV_CMD),
    ];
    let unchecked_commands = [
        "config prof ls -v",
        "proj ls -v",
        "env ls -v",
        "completions bash",
    ];
    cloudtruth!("proj ls -f csv")
        .assert()
        .success()
        .stdout(not(contains(proj.name())));
    cloudtruth!("env ls -f csv")
        .assert()
        .success()
        .stdout(not(contains(env.name())));

    for cmd in checked_commands {
        cloudtruth!("--project {proj} --env {env} {cmd}")
            .assert()
            .failure()
            .stderr(
                contains!("The '{proj}' project could not be found in your account.").and(
                    contains!("The '{env}' environment could not be found in your account."),
                ),
            );
    }
    for cmd in unchecked_commands {
        cloudtruth!("--project {proj} --env {env} {cmd}")
            .assert()
            .success();
    }
    // Create project and then auto-delete when it leaves this scope
    proj.clone().with_scope(|proj| {
        // Project present, missing environment
        for cmd in checked_commands {
            cloudtruth!("--project {proj} --env {env} {cmd}")
                .assert()
                .failure()
                .stderr(
                    not(contains!(
                        "The '{proj}' project could not be found in your account."
                    ))
                    .and(contains!(
                        "The '{env}' environment could not be found in your account."
                    )),
                );
        }
    });

    // environment present, missing project
    let env = env.create();
    for cmd in checked_commands {
        cloudtruth!("--project {proj} --env {env} {cmd}")
            .assert()
            .failure()
            .stderr(
                contains!("The '{proj}' project could not be found in your account.").and(not(
                    contains!("The '{env}' environment could not be found in your account."),
                )),
            );
    }

    // both present
    let proj = proj.create();
    for cmd in checked_commands {
        cloudtruth!("--project {proj} --env {env} {cmd}")
            .assert()
            .success();
    }
}

#[integration_test]
fn test_arg_configurable_timeout() {
    cloudtruth!("project ls -v")
        .env(CT_REQ_TIMEOUT, "0")
        .assert()
        .failure()
        .stderr(contains("timed out"));
}

#[integration_test]
fn test_arg_invalid_server() {
    cloudtruth!("project ls -v")
        .env(CT_SERVER_URL, "0.0.0.0:0")
        .assert()
        .failure()
        .stderr(contains("relative URL without a base"));
    cloudtruth!("project ls -v")
        .env(CT_SERVER_URL, "https://0.0.0.0:0/graphql")
        .assert()
        .failure()
        .stderr(contains("error trying to connect"));
}

#[integration_test]
fn test_arg_authentication_errors() {
    let cmds = [
        "env ls -v",
        "param ls -v",
        "proj ls -v",
        "int ex -v",
        "int ls -v",
        formatcp!("run -i none -- {}", DISPLAY_ENV_CMD),
    ];
    for cmd in cmds {
        cloudtruth!("--api-key abc123 {cmd}")
            .assert()
            .failure()
            .stderr(
                contains("Not Authenticated").and(contains("Incorrect authentication credentials")),
            );
    }
}

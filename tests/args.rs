use cloudtruth_config::CT_PROJECT;
use integration_test_harness::prelude::*;

#[integration_test]
fn test_args_missing_subcommands() {
    let proj = ScopedProject::with_prefix("missing-subarg");
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
                .env(CT_PROJECT, &proj)
                .assert()
                .success()
                .stderr(contains!("WARN: No '{subcmd}' sub-command executed."));
        }
    }
}

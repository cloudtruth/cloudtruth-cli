use color_eyre::eyre::Result;

fn main() -> Result<()> {
    cloudtruth::main()
}

#[cfg(test)]
mod main_test {
    use cloudtruth_test_harness::prelude::*;

    use cloudtruth_config::{CT_API_KEY, CT_PROFILE};

    #[test]
    #[use_harness]
    fn missing_subcommands() {
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
            // (
            //     "parameters",
            //     vec!["parameter", "params", "param", "par", "pa", "p"],
            // ),
            ("parameter-types", vec!["param-type", "types", "type", "ty"]),
            ("projects", vec!["project", "proj"]),
            // ("run", vec!["ru", "r"]),
            // ("templates", vec!["template", "te", "t"]),
            ("users", vec!["user", "us", "u"]),
            ("users invitations", vec!["user invitation", "us in", "u i"]),
            ("versions", vec!["version", "vers", "ver", "ve", "v"]),
        ];
        for (subcmd, aliases) in subcmds {
            for alias in core::iter::once(subcmd).chain(aliases) {
                cloudtruth!("{alias}")
                    .offline_env()
                    .env(CT_API_KEY, "dummy-key")
                    .env(CT_PROFILE, "default")
                    .assert()
                    .success()
                    .stderr(contains!("WARN: No '{subcmd}' sub-command executed."));
            }
        }
    }

    #[test]
    #[use_harness]
    fn requires_at_least_one_subcommand() {
        // Verify that invoking the CLI app without any arguments sets an error status code and
        // prints out the help message.
        let no_subcmd = cloudtruth!("").offline_env().assert().failure();
        let help_cmd = cloudtruth!("help").offline_env().assert().success();
        let help_message = std::str::from_utf8(&help_cmd.get_output().stdout)
            .unwrap()
            .to_string();
        no_subcmd.stderr(diff(help_message));
    }

    #[test]
    #[use_harness]
    fn completions_work_without_config() {
        cloudtruth!("completions bash")
            .offline_env()
            .assert()
            .success();
    }

    #[test]
    #[use_harness]
    fn completions_error_with_bad_shell_name() {
        cloudtruth!("completions bad")
            .offline_env()
            .assert()
            .failure()
            .stderr(contains("'bad' isn't a valid value"));
    }

    #[test]
    #[use_harness]
    fn need_api_key() {
        let commands = &[
            "parameters list",
            "environments list",
            "integrations list",
            "templates list",
            "--env non-default templates list",
            "run --command printenv",
            "run -c printenv",
            "run -s FOO=BAR -- ls -lh /tmp",
        ];
        for cmd_args in commands {
            println!("need_api_key test: {}", cmd_args);
            cloudtruth!("{cmd_args}")
                .offline_env()
                .env(CT_API_KEY, "")
                .env(CT_PROFILE, "default")
                .assert()
                .failure()
                .stderr(starts_with("The API key is missing."));
        }
    }

    #[test]
    #[use_harness]
    fn missing_profile() {
        let commands = ["projects", "environments", "integrations"];
        for cmd_args in commands {
            let prof_name = "no-prof-with-this-name";
            println!("missing_profile test: {}", cmd_args);
            let warn_msg =
                format!("Profile '{prof_name}' does not exist in your configuration file");
            cloudtruth!("{cmd_args}")
                .offline_env()
                .env(CT_API_KEY, "dummy-key")
                .env(CT_PROFILE, prof_name)
                .assert()
                .stderr(contains(warn_msg));
        }
    }

    #[test]
    fn help_text() {
        trycmd::TestCases::new()
            .register_bin("cloudtruth", cli_bin_path!())
            .case("examples/help-text/*.md");
    }
}

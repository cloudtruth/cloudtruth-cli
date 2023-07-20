use cloudtruth_config::{CT_ENVIRONMENT, CT_PROJECT};
use cloudtruth_test_harness::prelude::*;
use indoc::{formatdoc, indoc};
use maplit::hashmap;

#[test]
#[use_harness]
fn test_parameters_basic_empty_list() {
    let proj = Project::with_prefix("param-empty-list").create();
    // check that there are no parameters
    cloudtruth!("--project {proj} parameters list")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("--project {proj} parameters list --values")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("--project {proj} parameters list --values --secrets")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
}

#[test]
#[use_harness]
fn test_parameters_basic() {
    let proj = Project::with_prefix("param-basic").create();

    // add parameter
    cloudtruth!(
        "--project {proj} parameters set my_param \
        --value 'cRaZy value' \
        --desc 'this is just a test description'"
    )
    .assert()
    .success();

    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(diff(
            "\
            +----------+-------------+---------+------------+-------+----------+--------+---------------------------------+\n\
            | Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |\n\
            +----------+-------------+---------+------------+-------+----------+--------+---------------------------------+\n\
            | my_param | cRaZy value | default | string     | 0     | internal | false  | this is just a test description |\n\
            +----------+-------------+---------+------------+-------+----------+--------+---------------------------------+\n\
            "
        ));

    // use CSV
    cloudtruth!("--project {proj} parameters ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(""))
        .stdout(diff(
            "\
            Name,Value,Source,Param Type,Rules,Type,Secret,Description\n\
            my_param,cRaZy value,default,string,0,internal,false,this is just a test description\n\
            ",
        ));

    // get the parameter
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("cRaZy value"));

    // get the parameter details
    cloudtruth!("--project {proj} parameters get my_param --details")
        .assert()
        .success()
        .stdout(contains_all!(
            "Name: my_param",
            "Value: cRaZy value",
            "Source: default",
            "Secret: false",
            "Description: this is just a test description",
        ));

    // idempotent
    cloudtruth!("--project {proj} parameters set my_param --value 'cRaZy value'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(contains_all!(
            "my_param",
            "cRaZy value",
            "this is just a test description",
        ));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("cRaZy value"));

    // update just the value
    cloudtruth!("--project {proj} parameters set my_param --value 'new_value'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(contains_all!(
            "my_param",
            "new_value",
            "this is just a test description",
        ));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("new_value"));

    // update the description
    cloudtruth!("--project {proj} parameters set my_param -d 'alt description'")
        .assert()
        .success();
}

#[test]
#[use_harness]
fn test_parameters_basic_delete() {
    let proj = Project::with_prefix("param-delete").create();
    // add parameter
    cloudtruth!("--project {proj} parameters set my_param")
        .assert()
        .success();

    // delete
    cloudtruth!("--project {proj} parameters delete --yes my_param")
        .assert()
        .success()
        .stdout(contains("Removed parameter 'my_param'"));
    cloudtruth!("--project {proj} parameters list --values --secrets")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));

    // try to delete again
    cloudtruth!("--project {proj} parameters delete -y my_param")
        .assert()
        .success();
}

#[test]
#[use_harness]
fn test_parameters_basic_no_update() {
    let proj = Project::with_prefix("param-no-update").create();

    cloudtruth!(
        "--project {proj} parameters set my_param \
        --value my_value \
        --desc 'test description'"
    )
    .assert()
    .success();
    // no update
    cloudtruth!("--project {proj} parameters set my_param")
        .assert()
        .success()
        .stdout(contains("Updated parameter 'my_param'"));
    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(contains_all!("my_param", "my_value", "test description"));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("my_value"));
}

#[test]
#[use_harness]
fn test_paramer_basic_rename() {
    let proj = Project::with_prefix("param-rename").create();

    // add parameter
    cloudtruth!(
        "--project {proj} parameters set my_param \
        --value my_value \
        --desc 'parameter rename test'"
    )
    .assert()
    .success();
    // rename
    cloudtruth!("--project {proj} parameters set my_param -r renamed_param")
        .assert()
        .success()
        .stdout(contains("Updated parameter 'renamed_param'"));
    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(contains_all!(
            "renamed_param",
            "value",
            "parameter rename test",
        ));
}

#[test]
#[use_harness]
fn test_parameters_basic_no_value() {
    let proj = Project::with_prefix("param-basic-no-value").create();

    // create a parameter with no value
    cloudtruth!("--project {proj} parameters set no_value")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters list --values -f csv")
        .assert()
        .success()
        .stdout(contains("no_value,-,,string,0,internal,false"));
}

#[test]
#[use_harness]
fn test_parameters_basic_conflicting_options() {
    let proj = Project::with_prefix("param-basic-conflicting-opts").create();
    // make sure we error out on conflicting arguments
    cloudtruth!("--project {proj} parameters list --rules --external")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --rules --evaluated")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --external --evaluated")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --parents --external")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --parents --evaluated")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --parents --rules")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --children --parents")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --children --rules")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --children --external")
        .assert()
        .stderr(contains("are mutually exclusive"));
    cloudtruth!("--project {proj} parameters list --children --evaluated")
        .assert()
        .stderr(contains("are mutually exclusive"));
}

#[test]
#[use_harness]
fn test_parameters_basic_secret_list() {
    let proj = Project::with_prefix("param-secret").create();
    cloudtruth!("--project {proj} parameters list")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("--project {proj} parameters list --values ")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("--project {proj} parameters list --values --secrets")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("--project {proj} parameters set my_param --secret true --value super-SENSITIVE-vAluE --desc 'my secret value'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(diff(
            "\
            +----------+-------+---------+------------+-------+----------+--------+-----------------+\n\
            | Name     | Value | Source  | Param Type | Rules | Type     | Secret | Description     |\n\
            +----------+-------+---------+------------+-------+----------+--------+-----------------+\n\
            | my_param | ***** | default | string     | 0     | internal | true   | my secret value |\n\
            +----------+-------+---------+------------+-------+----------+--------+-----------------+\n\
            ",
        ));
    cloudtruth!("--project {proj} parameters ls -v -f csv")
        .assert()
        .success()
        .stdout(diff(
            "\
            Name,Value,Source,Param Type,Rules,Type,Secret,Description\n\
            my_param,*****,default,string,0,internal,true,my secret value\n\
            ",
        ));
    cloudtruth!("--project {proj} parameters list --values --secrets")
        .assert()
        .success()
        .stdout(diff(
            "\
            +----------+-----------------------+---------+------------+-------+----------+--------+-----------------+\n\
            | Name     | Value                 | Source  | Param Type | Rules | Type     | Secret | Description     |\n\
            +----------+-----------------------+---------+------------+-------+----------+--------+-----------------+\n\
            | my_param | super-SENSITIVE-vAluE | default | string     | 0     | internal | true   | my secret value |\n\
            +----------+-----------------------+---------+------------+-------+----------+--------+-----------------+\n\
            "
        ));
    cloudtruth!("--project {proj} parameters list --values --secrets --format csv")
        .assert()
        .success()
        .stdout(diff(
            "\
            Name,Value,Source,Param Type,Rules,Type,Secret,Description\n\
            my_param,super-SENSITIVE-vAluE,default,string,0,internal,true,my secret value\n\
            ",
        ));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("super-SENSITIVE-vAluE"));
    cloudtruth!("--project {proj} parameters get my_param --details")
        .assert()
        .success()
        .stdout(contains_all!(
            "Name: my_param",
            "Value: super-SENSITIVE-vAluE",
            "Source: default",
            "Secret: true",
            "Description: my secret value",
        ));
}

#[test]
#[use_harness]
fn test_parameters_basic_secret_idempotent() {
    let proj = Project::with_prefix("param-secret-idempotent").create();
    cloudtruth!("--project {proj} parameters set my_param --secret true --value super-SENSITIVE-vAluE --desc 'my secret value'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters set my_param --value super-SENSITIVE-vAluE")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v -s")
        .assert()
        .success()
        .stdout(contains_all!(
            "my_param",
            "super-SENSITIVE-vAluE",
            "my secret value",
        ));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("super-SENSITIVE-vAluE"));
}

#[test]
#[use_harness]
fn test_parameters_basic_secret_update() {
    let proj = Project::with_prefix("param-secret-update").create();
    cloudtruth!("--project {proj} parameters set my_param --secret true --value super-SENSITIVE-vAluE --desc 'my secret value'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters set my_param --value new_value")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v -s")
        .assert()
        .success()
        .stdout(contains_all!("my_param", "new_value", "my secret value"));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("new_value"));
    cloudtruth!("--project {proj} parameters set my_param -d 'alt description'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v -s")
        .assert()
        .success()
        .stdout(contains_all!("my_param", "new_value", "alt description"));
    cloudtruth!("--project {proj} parameters get my_param")
        .assert()
        .success()
        .stdout(contains("new_value"));
}

#[test]
#[use_harness]
fn test_parameters_basic_secret_delete() {
    let proj = Project::with_prefix("param-secret-delete").create();
    cloudtruth!("--project {proj} parameters set my_param --secret true --value super-SENSITIVE-vAluE --desc 'my secret value'")
    .assert()
    .success();
    cloudtruth!("--project {proj} parameters delete my_param -y")
        .assert()
        .success()
        .stdout(contains("Removed parameter 'my_param'"));
    cloudtruth!("--project {proj} parameters list --values --secrets")
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("--project {proj} parameters delete my_param -y")
        .assert()
        .success()
        .stdout(contains("Did not find parameter 'my_param'"));
}

#[test]
#[use_harness]
fn test_parameters_basic_secret_no_value() {
    let proj = Project::with_prefix("param-secret-no-value").create();
    cloudtruth!("--project {proj} parameters set my_param --secret true")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters list --values -f csv")
        .assert()
        .success()
        .stdout(contains("my_param,-,,string,0,internal,true"));
}

#[test]
#[use_harness]
fn test_parameters_copy() {
    let proj = Project::with_prefix("param-copy").create();
    cloudtruth!("--project {proj} parameters set param-src --value my-value")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters cp param-src param-dest")
        .assert()
        .success()
        .stdout(contains!(
            "Copied parameter 'param-src' to 'param-dest' in project '{proj}'"
        ));
    cloudtruth!("--project {proj} parameters ls -v ")
        .assert()
        .success()
        .stdout(contains_all!("param-src", "param-dest", "my-value"));
}

#[test]
fn test_parameters_export() -> Result<()> {
    let proj = Project::with_prefix("param-export").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_export.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_generate() -> Result<()> {
    let proj = Project::with_prefix("param-generate").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_generate.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_integration_errors() -> Result<()> {
    let proj = Project::with_prefix("param-integration-errors").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_integration_errors.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_names() -> Result<()> {
    let proj = Project::with_prefix("param-names").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_names.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_over_specified() -> Result<()> {
    let proj = Project::with_prefix("param-overspecified").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_over_specified.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_rules_bool() -> Result<()> {
    let proj = Project::with_prefix("param-rules-bool").create();
    let env = Environment::with_prefix("param-rules-bool").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_rules_bool.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .env(CT_ENVIRONMENT, env.to_name())
        .insert_var("[PROJECT]", proj.to_name())?
        .insert_var("[ENV]", env.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_rules_string() -> Result<()> {
    let proj = Project::with_prefix("param-rules-string").create();
    let env = Environment::with_prefix("param-rules-string").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_rules_string.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .env(CT_ENVIRONMENT, env.to_name())
        .insert_var("[PROJECT]", proj.to_name())?
        .insert_var("[ENV]", env.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_secret_switch() -> Result<()> {
    let proj = Project::with_prefix("param-rules-secret-switch").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_secret_switch.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_table_formats() -> Result<()> {
    let proj = Project::with_prefix("param-rules-table-formats").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_table_formats.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
fn test_parameters_types() -> Result<()> {
    let proj = Project::with_prefix("param-types").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_types.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

#[test]
#[use_harness]
fn test_parameters_project_separation() {
    let proj1 = Project::with_prefix("proj-separation1").create();
    let proj2 = Project::with_prefix("proj-separation2").create();

    cloudtruth!("param set sna --value foo")
        .env(CT_PROJECT, proj1.name())
        .assert()
        .success();
    cloudtruth!("param set sensitive --value classified --secret true")
        .env(CT_PROJECT, proj1.name())
        .assert()
        .success();
    cloudtruth!("param set sna --value fu")
        .env(CT_PROJECT, proj2.name())
        .assert()
        .success();
    cloudtruth!("param set sensitive --value top-secret --secret true")
        .env(CT_PROJECT, proj2.name())
        .assert()
        .success();
    cloudtruth!("--project {proj1} param ls -v -s")
        .assert()
        .success()
        .stdout(diff(
            "\
    +-----------+------------+---------+------------+-------+----------+--------+-------------+\n\
    | Name      | Value      | Source  | Param Type | Rules | Type     | Secret | Description |\n\
    +-----------+------------+---------+------------+-------+----------+--------+-------------+\n\
    | sensitive | classified | default | string     | 0     | internal | true   |             |\n\
    | sna       | foo        | default | string     | 0     | internal | false  |             |\n\
    +-----------+------------+---------+------------+-------+----------+--------+-------------+\n\
    ",
        ));
    cloudtruth!("--project {proj2} param ls -v -s")
        .assert()
        .success()
        .stdout(diff(
            "\
    +-----------+------------+---------+------------+-------+----------+--------+-------------+\n\
    | Name      | Value      | Source  | Param Type | Rules | Type     | Secret | Description |\n\
    +-----------+------------+---------+------------+-------+----------+--------+-------------+\n\
    | sensitive | top-secret | default | string     | 0     | internal | true   |             |\n\
    | sna       | fu         | default | string     | 0     | internal | false  |             |\n\
    +-----------+------------+---------+------------+-------+----------+--------+-------------+\n\
    ",
        ));
    cloudtruth!("--project {proj1} param export docker -s")
        .assert()
        .success()
        .stdout(diff(indoc! {"
            SENSITIVE=classified
            SNA=foo

        "}));
    cloudtruth!("--project {proj2} param export docker -s")
        .assert()
        .success()
        .stdout(diff(indoc! {"
            SENSITIVE=top-secret
            SNA=fu

        "}));
}

#[test]
#[use_harness]
fn test_parameters_environment_separation() {
    let proj = Project::with_prefix("env-separation").create();
    // env1 is "default"
    let env2 = Environment::with_prefix("env-separation2").create();
    let env3 = Environment::with_prefix("env-separation3")
        .parent(&env2)
        .create();
    let envs = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };
    cloudtruth!("param set base --value first")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set pitch --value slider")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("base,first,default").and(contains!("pitch,slider,default")));
    cloudtruth!("--env {env2} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("base,first,default").and(contains!("pitch,slider,default")));
    cloudtruth!("--env {env3} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("base,first,default").and(contains!("pitch,slider,default")));
    cloudtruth!("param env 'no-such-parameter' -f csv")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Parameter 'no-such-parameter' was not found"));
    cloudtruth!("param env base -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("default,first,,").and(not(contains(env2.name()))));
    cloudtruth!("param env base -f csv --all")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("default,first,,").and(contains!("{env2},-,,")));
    cloudtruth!("--env {env2} param set base --value second")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("--env {env2} param set pitch --value split --secret true")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("--env {env3} param set base --value third")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("--env {env3} param set pitch --value heater --secret true")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param environment 'pitch' -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(
            contains!("default,*****,,")
                .and(contains!("{env2},*****,,"))
                .and(contains!("{env3},*****,,")),
        );
    cloudtruth!("param environment 'pitch' -f csv -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(
            contains!("default,slider,,")
                .and(contains!("{env2},split,,"))
                .and(contains!("{env3},heater,,")),
        );
    cloudtruth!("param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("base,first,default").and(contains("pitch,slider,default")));
    cloudtruth!("--env {env2} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("base,second,{env2}").and(contains!("pitch,split,{env2}")));
    cloudtruth!("--env {env3} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("base,third,{env3}").and(contains!("pitch,heater,{env3}")));
    cloudtruth!("param export docker -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(indoc! {"
            BASE=first
            PITCH=slider
            
        "}));
    cloudtruth!("--env {env2} param export docker -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(indoc! {"
            BASE=second
            PITCH=split
            
        "}));
    cloudtruth!("--env {env3} param export docker -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(indoc! {"
            BASE=third
            PITCH=heater
            
        "}));
    cloudtruth!("--env {env2} param unset base")
        .envs(&envs)
        .assert()
        .success()
        .stdout(
            contains("Removed parameter value 'base'").and(contains!("for environment '{env2}'")),
        );
    cloudtruth!("--env {env3} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("base,third,{env3}"));
    cloudtruth!("--env {env2} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("base,first,default"));
    cloudtruth!("param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("base,first,default"));
    cloudtruth!("--env {env3} param unset base")
        .envs(&envs)
        .assert()
        .success()
        .stdout(
            contains("Removed parameter value 'base'").and(contains!("for environment '{env3}'")),
        );
    cloudtruth!("--env {env3} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("base,first,default"));
    cloudtruth!("--env {env2} param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("base,first,default"));
    cloudtruth!("param ls -v -s -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("base,first,default"));
}

#[test]
#[use_harness]
fn test_parameters_local_file() {
    let file = TestFile::with_contents("static val from file").unwrap();
    let proj = Project::with_prefix("local-file").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };
    cloudtruth!("parameters list --values --secrets")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("param set my_param --input {file} --desc 'param set from file input'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(indoc! {"
            +----------+----------------------+---------+------------+-------+----------+--------+---------------------------+
            | Name     | Value                | Source  | Param Type | Rules | Type     | Secret | Description               |
            +----------+----------------------+---------+------------+-------+----------+--------+---------------------------+
            | my_param | static val from file | default | string     | 0     | internal | false  | param set from file input |
            +----------+----------------------+---------+------------+-------+----------+--------+---------------------------+
        "}));
    cloudtruth!("param set my_param --value update-from-value")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(indoc! {"
            +----------+-------------------+---------+------------+-------+----------+--------+---------------------------+
            | Name     | Value             | Source  | Param Type | Rules | Type     | Secret | Description               |
            +----------+-------------------+---------+------------+-------+----------+--------+---------------------------+
            | my_param | update-from-value | default | string     | 0     | internal | false  | param set from file input |
            +----------+-------------------+---------+------------+-------+----------+--------+---------------------------+
        "}));
    let file = TestFile::with_contents("another-static-file").unwrap();
    cloudtruth!("param set my_param --input '{file}'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(indoc! {"
            +----------+---------------------+---------+------------+-------+----------+--------+---------------------------+
            | Name     | Value               | Source  | Param Type | Rules | Type     | Secret | Description               |
            +----------+---------------------+---------+------------+-------+----------+--------+---------------------------+
            | my_param | another-static-file | default | string     | 0     | internal | false  | param set from file input |
            +----------+---------------------+---------+------------+-------+----------+--------+---------------------------+
        "}));
}

#[test]
#[use_harness]
fn test_parameters_project_inheritance() {
    let parent = Project::with_prefix("param-parent").create();
    let child1 = Project::with_prefix("param-child1")
        .parent(&parent)
        .create();
    let child2 = Project::with_prefix("param-child2")
        .parent(&parent)
        .create();

    cloudtruth!("--project {parent} params ls -v --children")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a child project found in project",
        ));
    cloudtruth!("--project {child1} params ls -v --children")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a child project found in project",
        ));
    cloudtruth!("--project {child2} params ls -v --children")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a child project found in project",
        ));
    cloudtruth!("--project {child1} param set param1 --value some_value")
        .assert()
        .success();
    cloudtruth!("--project {child1} param set secret2 --value ssshhhh --secret true")
        .assert()
        .success();
    cloudtruth!("--project {parent} param ls -v")
        .assert()
        .success()
        .stdout(contains("No parameters found in project"));
    cloudtruth!("--project {child1} param ls -v")
        .assert()
        .success()
        .stdout(contains_all!("param1", "secret2"));
    cloudtruth!("--project {child2} param ls -v")
        .assert()
        .success()
        .stdout(contains("No parameters found in project"));
    cloudtruth!("--project {parent} params ls -v --parents")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a parent project found in project",
        ));
    cloudtruth!("--project {child1} params ls -v --parents")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a parent project found in project",
        ));
    cloudtruth!("--project {child2} params ls -v --parents")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a parent project found in project",
        ));
    cloudtruth!("--project {parent} params ls -v -f csv --children")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param1,some_value,{child1}
            secret2,*****,{child1}
        "}));
    cloudtruth!("--project {child1} params ls -v -f csv --children")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a child project found in project",
        ));
    cloudtruth!("--project {child2} params ls -v -f csv --children")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a child project found in project",
        ));
    cloudtruth!("--project {parent} param set param3 --value some_value")
        .assert()
        .success();
    cloudtruth!("--project {parent} param set secret4 --value 'be vewy vewy quiet' --secret true")
        .assert()
        .success();
    cloudtruth!("--project {parent} params ls -v -f csv --parents")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a parent project found in project",
        ));
    cloudtruth!("--project {child1} params ls -v -f csv --parents")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param3,some_value,{parent}
            secret4,*****,{parent}
        "}));
    cloudtruth!("--project {child2} params ls -v -f csv --parents")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param3,some_value,{parent}
            secret4,*****,{parent}
        "}));
    cloudtruth!("--project {parent} params ls -v -s -f csv --parents")
        .assert()
        .success()
        .stdout(contains(
            "No parameters from a parent project found in project",
        ));
    cloudtruth!("--project {child1} params ls -v -s -f csv --parents")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param3,some_value,{parent}
            secret4,be vewy vewy quiet,{parent}
        "}));
    cloudtruth!("--project {child2} params ls -v -s -f csv --parents")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param3,some_value,{parent}
            secret4,be vewy vewy quiet,{parent}
        "}));
    let grandchild = Project::with_prefix("params-grandchild")
        .parent(&child1)
        .create();
    cloudtruth!("--project {child1} params ls -v -s -f csv --parents")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param3,some_value,{parent}
            secret4,be vewy vewy quiet,{parent}
        "}));
    cloudtruth!("--project {grandchild} param set param5 --value grand")
        .assert()
        .success();
    cloudtruth!(
        "--project {grandchild} param set secret6 --value 'im hunting wabbits' --secret true"
    )
    .assert()
    .success();
    cloudtruth!("--project {parent} params ls -v -f csv --children")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param1,some_value,{child1}
            secret2,*****,{child1}
            param5,grand,{grandchild}
            secret6,*****,{grandchild}
        "}));
    cloudtruth!("--project {child1} params ls -v -f csv --children")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
            Name,Value,Project
            param5,grand,{grandchild}
            secret6,*****,{grandchild}
        "}));
    cloudtruth!("--project {grandchild} param del -y param1")
        .assert()
        .failure()
        .stderr(contains!(
            "Parameter 'param1' must be deleted from project '{child1}'"
        ));
    cloudtruth!("--project {grandchild} param set param1 -d 'new desc'")
        .assert()
        .failure()
        .stderr(contains!(
            "Parameter 'param1' must be set from project '{child1}'"
        ));
    cloudtruth!("--project {grandchild} param set param1 -v 'next value'")
        .assert()
        .failure()
        .stderr(contains!(
            "Parameter 'param1' must be set from project '{child1}'"
        ));
    cloudtruth!("--project {child2} param set param5 --value slam")
        .assert()
        .success();
    cloudtruth!("--project {child2} param set secret6 --value 'kill the wabbit'") //NOTE: not a secret
        .assert()
        .success();
    cloudtruth!("--project {parent} params ls -v -f csv --children")
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Name,Value,Project
            param1,some_value,{child1}
            secret2,*****,{child1}
            param5,grand,{grandchild}
            secret6,*****,{grandchild}
            param5,slam,{child2}
            secret6,kill the wabbit,{child2}
        "}));
}

#[test]
#[use_harness]
fn test_parameters_pagination() {
    const PAGE_SIZE: usize = 5;
    let proj = Project::with_prefix("param-pagination").create();
    for i in 0..=PAGE_SIZE {
        cloudtruth!("param set param{i}")
            .env(CT_PROJECT, proj.name())
            .assert()
            .success();
    }
    cloudtruth!("param ls")
        .env(CT_PROJECT, proj.name())
        .rest_debug()
        .page_size(PAGE_SIZE)
        .assert()
        .success()
        .paginated(PAGE_SIZE);
}

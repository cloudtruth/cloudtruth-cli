use std::borrow::Cow;

use chrono::{DateTime, Utc};
use cloudtruth_config::{CT_ENVIRONMENT, CT_PROJECT};
use cloudtruth_test_harness::{
    output::parameter::{
        ParseParamDiffExt, ParseParamDriftExt, ParseParamEnvExt, ParseParamListExt,
    },
    prelude::*,
};
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
#[use_harness]
fn test_parameters_rules_string() {
    let proj = Project::with_prefix("string-rules").create();
    let env = Environment::with_prefix("string-rules").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name(),
        CT_ENVIRONMENT => env.name()
    };
    /* Create a basic parameter without a value, so the rule cannot be violated */
    cloudtruth!("param set param1 --value 'some value'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param unset param1")
        .envs(&envs)
        .assert()
        .success();
    /* See no rules */
    cloudtruth!("param list --rules -vf csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("No parameter rules found in project"));
    /* Create parameter with constraints */
    cloudtruth!("param set param1 --regex 'abc.*' --min-len 10 --max-len 15")
        .envs(&envs)
        .assert()
        .success();
    /* See the 2 rules are registered */
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("param1,-,,string,3,internal,false"));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            "param1,string,regex,abc.*",
            "param1,string,min-len,10",
            "param1,string,max-len,15"
        ));
    cloudtruth!("param list --rules")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff("param1\n"));
    /* test min and max */
    cloudtruth!("param set param1 -v aaaaaaaaa")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Value must be at least 10 characters"));
    cloudtruth!("param set param1 -v aaaaaaaaaaaaaaaa")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Value must be at most 15 characters"));
    cloudtruth!("param set param1 -v aaaaaaaaaaaaaaa")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Value does not match regular expression abc.*"));
    /* test middle value */
    cloudtruth!("param set param1 -v abcabcabcabc")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!(
            "param1,abcabcabcabc,{env},string,3,internal,false"
        ));
    /* Update the rules */
    cloudtruth!("param set param1 --min-len 5")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --max-len 30")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --regex 'a.*'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!(
            "param1,abcabcabcabc,{env},string,3,internal,false"
        ));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            "param1,string,regex,a.*",
            "param1,string,min-len,5",
            "param1,string,max-len,30"
        ));
    /* Remove the rules */
    cloudtruth!("param set param1 --no-regex")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!(
            "param1,abcabcabcabc,{env},string,2,internal,false"
        ));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            "param1,string,min-len,5",
            "param1,string,max-len,30"
        ));
    cloudtruth!("param set param1 --no-max-len")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!(
            "param1,abcabcabcabc,{env},string,1,internal,false"
        ));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("param1,string,min-len,5"));
    cloudtruth!("param set param1 --no-min-len")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!(
            "param1,abcabcabcabc,{env},string,0,internal,false"
        ));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("No parameter rules found in project"));
    /* Failed create rules while values in place */
    cloudtruth!("param set param1 --min-len 15")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule create error: Rule may not be applied to param1",
            "Value must be at least 15 characters"
        ));
    cloudtruth!("param set param1 --max-len 10")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule create error: Rule may not be applied to param1",
            "Value must be at most 10 characters"
        ));
    cloudtruth!("param set param1 --min-len 2")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --min-len 15")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule update error: Rule may not be applied to param1",
            "Value must be at least 15 characters"
        ));
    cloudtruth!("param set param1 --max-len 22")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --max-len 10")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule update error: Rule may not be applied to param1",
            "Value must be at most 10 characters"
        ));
    /* Delete the rules */
    cloudtruth!("param set param1 --no-min-len")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --no-max-len")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --no-regex")
        .envs(&envs)
        .assert()
        .success();
    /* Error cases */
    cloudtruth!("param set param1 --max 10 --min -1")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "max rules not valid for string parameters",
            "min rules not valid for string parameters"
        ));
    cloudtruth!("param set param1 --max -10")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("max rules not valid for string parameters"));
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!(
            "param1,abcabcabcabc,{env},string,0,internal,false"
        ));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("No parameter rules found in project"));
    /* See we don't leave any parameter behind when creating a parameter with an invalid rule */
    cloudtruth!("param del -y param1")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --type string --value 9 --max 10")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("max rules not valid for string parameters"));
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
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

#[test]
#[use_harness]
fn test_parameters_as_of_tag() {
    let proj = Project::with_prefix("param-tags").create();
    let env = Environment::with_prefix("param-tags").create();
    let mut envs = hashmap! {
        CT_PROJECT => proj.name().as_str(),
        CT_ENVIRONMENT => env.name().as_str()
    };
    cloudtruth!("param set param1 --value original")
        .envs(&envs)
        .assert()
        .success();
    let param1 = cloudtruth!("param list --show-times -f json")
        .envs(&envs)
        .assert()
        .success()
        .get_param("param1")
        .unwrap();
    cloudtruth!("param set param1 --value updated")
        .envs(&envs)
        .assert()
        .success();
    let param2 = cloudtruth!("param list --show-times -f json")
        .envs(&envs)
        .assert()
        .success()
        .get_param("param1")
        .unwrap();
    cloudtruth!("env tag set {env} my-tag --desc 'quick tag'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --value final")
        .envs(&envs)
        .assert()
        .success();
    let result = cloudtruth!("param list --show-times -f json --as-of my-tag")
        .envs(&envs)
        .assert()
        .success()
        .get_param("param1")
        .unwrap();
    assert_eq!(result, param2);
    let modified_at = param1.modified_at.as_deref().unwrap();
    let result = cloudtruth!("param list --show-times -f json --as-of '{modified_at}'")
        .envs(&envs)
        .assert()
        .success()
        .get_param("param1")
        .unwrap();
    assert_eq!(result, param1);
    cloudtruth!("env tag set {env} my-tag --time '2021-01-20'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param diff --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param env param1 --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param export docker --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param get param1 --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param ls -v --as-of no-such-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `no-such-tag` could not be found in environment `{env}`"
        ));
    cloudtruth!("param diff --as-of no-such-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `no-such-tag` could not be found in environment `{env}`"
        ));
    cloudtruth!("param env param1 --as-of no-such-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `no-such-tag` could not be found in environment `{env}`"
        ));
    cloudtruth!("param export docker --as-of no-such-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `no-such-tag` could not be found in environment `{env}`"
        ));
    cloudtruth!("param get param1 --as-of no-such-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `no-such-tag` could not be found in environment `{env}`"
        ));
    envs.insert(CT_ENVIRONMENT, "default");
    cloudtruth!("param ls -v --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains(
            "Tag `my-tag` could not be found in environment `default`",
        ));
    cloudtruth!("param diff --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains(
            "Tag `my-tag` could not be found in environment `default`",
        ));
    cloudtruth!("param env param1 --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains(
            "Tag `my-tag` could not be found in environment `default`",
        ));
    cloudtruth!("param export docker --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains(
            "Tag `my-tag` could not be found in environment `default`",
        ));
    cloudtruth!("param get param1 --as-of my-tag")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains(
            "Tag `my-tag` could not be found in environment `default`",
        ));
}

#[test]
#[use_harness]
fn test_parameters_drift() {
    let proj = Project::with_prefix("param-drift").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };
    cloudtruth!(" param set PARAM1 --value my-param-value")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!(" param set PARAM2 --value ssssshhhhh --secret true")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!(" param set PARAM3 --value another-param-value")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!(" param set PARAM4 --value 'be vewwwwy qwiet' --secret true")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!(" param set PARAM5 --value vanilla")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!(" param set PARAM6 --value ssssshhhhhh --secret true")
        .envs(&envs)
        .assert()
        .success();
    let envs = hashmap! {
        CT_PROJECT => proj.name().as_str(),
        "PARAM1" => "my-param-value",
        "PARAM2" => "ssssshhhhh",
        "PARAM3" => "different in shell",
        "PARAM4" => "im hunting wabbits",
        "PARAM7" => "env-value"
    };
    cloudtruth!("param drift")
        .envs(&envs)
        .assert()
        .success()
        .stdout(not(contains_any!("PARAM1", "PARAM2")).and(
            contains_all!("PARAM3", "PARAM4", "PARAM5", "PARAM6", "PARAM7",).and(not(
                contains_any!(
                    "\nHOME\n",
                    "PWD",
                    "CLOUDTRUTH_PROFILE",
                    "CLOUDTRUTH_PROJECT",
                    "CLOUDTRUTH_ENVIRONMENT",
                    "CLOUDTRUTH_API_KEY"
                ),
            )),
        ));
    let json = cloudtruth!("param drift -f json")
        .env_clear()
        .envs(&envs)
        .assert()
        .success()
        .parse_param_drift();
    assert!(json.find_by_name("PARAM1").is_none());
    assert!(json.find_by_name("PARAM2").is_none());
    let entry = json.find_by_name("PARAM3").unwrap();
    assert_eq!(entry.difference, "changed");
    assert_eq!(entry.shell, "different in shell");
    assert_eq!(entry.cloudtruth, "another-param-value");
    let entry = json.find_by_name("PARAM3").unwrap();
    assert_eq!(entry.difference, "changed");
    assert_eq!(entry.shell, "different in shell");
    assert_eq!(entry.cloudtruth, "another-param-value");
    let entry = json.find_by_name("PARAM4").unwrap();
    assert_eq!(entry.difference, "changed");
    assert_eq!(entry.shell, "*****");
    assert_eq!(entry.cloudtruth, "*****");
    let entry = json.find_by_name("PARAM5").unwrap();
    assert_eq!(entry.difference, "removed");
    assert_eq!(entry.shell, "");
    assert_eq!(entry.cloudtruth, "vanilla");
    let entry = json.find_by_name("PARAM6").unwrap();
    assert_eq!(entry.difference, "removed");
    assert_eq!(entry.shell, "");
    assert_eq!(entry.cloudtruth, "*****");
    let entry = json.find_by_name("PARAM7").unwrap();
    assert_eq!(entry.difference, "added");
    assert_eq!(entry.shell, "env-value");
    assert_eq!(entry.cloudtruth, "");
    let json = cloudtruth!("param drift -sf json")
        .env_clear()
        .envs(&envs)
        .assert()
        .success()
        .parse_param_drift();
    assert!(json.find_by_name("PARAM1").is_none());
    assert!(json.find_by_name("PARAM2").is_none());
    let entry = json.find_by_name("PARAM3").unwrap();
    assert_eq!(entry.difference, "changed");
    assert_eq!(entry.shell, "different in shell");
    assert_eq!(entry.cloudtruth, "another-param-value");
    let entry = json.find_by_name("PARAM3").unwrap();
    assert_eq!(entry.difference, "changed");
    assert_eq!(entry.shell, "different in shell");
    assert_eq!(entry.cloudtruth, "another-param-value");
    let entry = json.find_by_name("PARAM4").unwrap();
    assert_eq!(entry.difference, "changed");
    assert_eq!(entry.shell, "im hunting wabbits");
    assert_eq!(entry.cloudtruth, "be vewwwwy qwiet");
    let entry = json.find_by_name("PARAM5").unwrap();
    assert_eq!(entry.difference, "removed");
    assert_eq!(entry.shell, "");
    assert_eq!(entry.cloudtruth, "vanilla");
    let entry = json.find_by_name("PARAM6").unwrap();
    assert_eq!(entry.difference, "removed");
    assert_eq!(entry.shell, "");
    assert_eq!(entry.cloudtruth, "ssssshhhhhh");
    let entry = json.find_by_name("PARAM7").unwrap();
    assert_eq!(entry.difference, "added");
    assert_eq!(entry.shell, "env-value");
    assert_eq!(entry.cloudtruth, "");

    let cloudtruth_cmd = cli_bin_path!().display();
    let json = cloudtruth!("run --permissive -c '{cloudtruth_cmd} param drift -vf json'")
        .envs(&envs)
        .assert()
        .success()
        .parse_param_drift();
    assert_eq!(
        json.len(),
        json.iter().filter(|p| p.difference == "added").count(),
    );
}

#[test]
#[use_harness]
fn test_parameters_diff() {
    let proj = Project::with_prefix("param-cmp").create();
    let env1 = Environment::with_prefix("param-left").create();
    let env2 = Environment::with_prefix("param-right").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };
    cloudtruth!("param list -vsf csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
    cloudtruth!("param set param1 --value some_value")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param set secret1 --value ssshhhh --secret true")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param diff -e {env1} --env {env2} -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,{env1},{env2}
            param1,some_value,-
            secret1,*****,-
        "}));
    cloudtruth!("param diff -e {env1} -e {env2} -f csv -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,{env1},{env2}
            param1,some_value,-
            secret1,ssshhhh,-
        "}));
    cloudtruth!("param set param1 --value different")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set secret1 --value 'be qwiet' --secret true")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param diff -e {env1} -e {env2} -f csv -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,{env1},{env2}
            param1,some_value,different
            secret1,ssshhhh,be qwiet
        "}));
    cloudtruth!("param diff -e {env1} -e {env2} -f csv -s -p value -p environment")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,{env1},{env2}
            param1,\"some_value,\n{env1}\",\"different,\ndefault\"
            secret1,\"ssshhhh,\n{env1}\",\"be qwiet,\ndefault\"
        "}));
    cloudtruth!("param set param1 --value matchers")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param set param1 --value matchers")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("param set secret1 --value 'im hunting wabbits'")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("param set secret1 --value 'im hunting wabbits'")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("param diff -e {env1} -e {env2} -f csv -s")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,{env1},{env2}
            secret1,ssshhhh,im hunting wabbits
        "}));
    let json = cloudtruth!("param diff -e {env1} -e {env2} -f json -p created-at -p modified-at")
        .envs(&envs)
        .assert()
        .parse_param_diff();
    let entry1 = &json[0];
    assert_eq!(entry1.parameter, "param1");
    let fields1 = entry1.split_fields(env1.name()).collect::<Vec<Cow<str>>>();
    let [created_at1, modified_at1, ..] = fields1.as_slice()
        else { panic!("Invalid timestamp fields in diff") };
    let fields2 = entry1.split_fields(env2.name()).collect::<Vec<Cow<str>>>();
    let [created_at2, modified_at2, ..] = fields2.as_slice()
        else { panic!("Invalid timestamp fields in diff") };
    assert!(created_at1.parse::<DateTime<Utc>>().is_ok());
    assert!(modified_at1.parse::<DateTime<Utc>>().is_ok());
    assert!(created_at2.parse::<DateTime<Utc>>().is_ok());
    assert!(modified_at2.parse::<DateTime<Utc>>().is_ok());
    let entry2 = &json[1];
    let fields1 = entry2.split_fields(env1.name()).collect::<Vec<Cow<str>>>();
    let [created_at1, modified_at1, ..] = fields1.as_slice()
        else { panic!("Invalid timestamp fields in diff") };
    let fields2 = entry2.split_fields(env2.name()).collect::<Vec<Cow<str>>>();
    let [created_at2, modified_at2, ..] = fields2.as_slice()
        else { panic!("Invalid timestamp fields in diff") };
    assert!(created_at1.parse::<DateTime<Utc>>().is_ok());
    assert!(modified_at1.parse::<DateTime<Utc>>().is_ok());
    assert!(created_at2.parse::<DateTime<Utc>>().is_ok());
    assert!(modified_at2.parse::<DateTime<Utc>>().is_ok());
    cloudtruth!("param diff -e {env1} -e {env2} --property fqn")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains(
            "No parameters or differences in compared properties found",
        ));
    cloudtruth!("param diff -f csv --as-of '{modified_at1}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,Current,{modified_at1}
            param1,different,-
            secret1,*****,-
        "}));
    cloudtruth!("param diff -f csv -s --as-of '{modified_at1}' --as-of '{modified_at2}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,{modified_at1},{modified_at2}
            param1,-,different
            secret1,-,be qwiet
        "}));
    cloudtruth!("param diff -f csv --as-of '{created_at2}' --as-of '{modified_at2}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains(
            "No parameters or differences in compared properties found.",
        ));
    cloudtruth!("param diff -f csv -e {env1} --as-of '{modified_at1}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff(formatdoc! {"
            Parameter,default,{env1} ({modified_at1})
            param1,different,some_value
        "}));
    cloudtruth!(
        "param diff -f csv -s -e {env1} --as-of {modified_at1} -e {env2} --as-of {modified_at2}"
    )
    .envs(&envs)
    .assert()
    .success()
    .stdout(diff(formatdoc! {"
            Parameter,{env1} ({modified_at1}),{env2} ({modified_at2})
            param1,some_value,matchers
            secret1,ssshhhh,im hunting wabbits
        "}));

    cloudtruth!("param difference")
        .envs(&envs)
        .assert()
        .success()
        .stderr(contains("Invalid comparing an environment to itself"));
    cloudtruth!("param difference -e {env1} -e {env1}")
        .envs(&envs)
        .assert()
        .success()
        .stderr(contains("Invalid comparing an environment to itself"));
    cloudtruth!("param difference --as-of 2021-08-27 --as-of 2021-08-27")
        .envs(&envs)
        .assert()
        .success()
        .stderr(contains("Invalid comparing an environment to itself"));
    cloudtruth!("param difference -e {env1} -e {env1} --as-of 2021-08-27 --as-of 2021-08-27")
        .envs(&envs)
        .assert()
        .success()
        .stderr(contains("Invalid comparing an environment to itself"));
    cloudtruth!("param difference -e 'charlie-foxtrot' -e {env2}")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Did not find environment 'charlie-foxtrot'"));
    cloudtruth!("param difference -e {env2} -e 'missing'")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Did not find environment 'missing'"));
    cloudtruth!("param difference -e env1 --env env2 -e env3")
        .envs(&envs)
        .assert()
        .success()
        .stderr(contains("Can specify a maximum of 2 environment values"));
    cloudtruth!("param difference --as-of 2021-08-01 --as-of 2021-08-02 --as-of 2021-08-03")
        .envs(&envs)
        .assert()
        .success()
        .stderr(contains("Can specify a maximum of 2 as-of values"));
}

#[test]
#[use_harness]
fn test_parameters_evaluated() {
    let proj = Project::with_prefix("evaluated").create();
    let env1 = Environment::with_prefix("eval1").create();
    let env2 = Environment::with_prefix("eval2").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };
    cloudtruth!("params set param1 --value 'first value'")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("params set param1 --value 'other value'")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("parameters list --evaluated")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("No evaluated parameters found in project {proj}"));
    cloudtruth!("parameters list --evaluated -v -s --show-times")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("No evaluated parameters found in project {proj}"));
    cloudtruth!("params set param2 --value my-value --evaluate true")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("params set param2 --value your-value")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("params list -v -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(
            contains!("param1,first value,{env1},string,0,internal,false").and(contains!(
                "param2,my-value,{env1},string,0,internal-evaluated,false"
            )),
        );
    cloudtruth!("params list -v -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success()
        .stdout(
            contains!("param1,other value,{env2},string,0,internal,false").and(contains!(
                "param2,your-value,{env2},string,0,internal,false"
            )),
        );
    cloudtruth!("param set param3 --value '{{{{ param1 }}}}' --evaluate true")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param set param3 --value '{{{{ param2 }}}}' --evaluate true")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("param get param3")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains("first value"));
    cloudtruth!("param list -v -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains_all!(
            format!("param1,first value,{env1},string,0,internal,false"),
            format!("param2,my-value,{env1},string,0,internal-evaluated,false"),
            format!("param3,first value,{env1},string,0,internal-evaluated,false")
        ));
    cloudtruth!("param list -v -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success()
        .stdout(contains_all!(
            format!("param1,other value,{env2},string,0,internal,false"),
            format!("param2,your-value,{env2},string,0,internal,false"),
            format!("param3,your-value,{env2},string,0,internal-evaluated,false")
        ));
    cloudtruth!("param set param4 --value '{{{{ param3 }}}}' --evaluate true")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param list -v -f csv --evaluated")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains!("param4,first value,{{{{ param3 }}}}"));

    cloudtruth!("param set param3 --value '{{{{ cloudtruth.parameters.unknown }}}}'")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .failure()
        .stderr(contains_all!(
            "Evaluation error",
            "contains references that do not exist",
            "unknown"
        ));
    cloudtruth!("param list -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains!(
            "param3,first value,{env1},string,0,internal-evaluated,false"
        ));

    cloudtruth!("param set param3 --evaluate false")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param list -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains_all!(
            format!("param1,first value,{env1},string,0,internal,false"),
            format!("param2,my-value,{env1},string,0,internal-evaluated,false"),
            format!("param3,{{{{ param1 }}}},{env1},string,0,internal,false"),
            format!("param4,{{{{ param1 }}}},{env1},string,0,internal-evaluated,false")
        ));
    cloudtruth!("param list -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success()
        .stdout(contains_all!(
            format!("param1,other value,{env2},string,0,internal,false"),
            format!("param2,your-value,{env2},string,0,internal,false"),
            format!("param3,your-value,{env2},string,0,internal-evaluated,false"),
            format!("param4,-,,string,0,internal,false")
        ));
    cloudtruth!("param get param3 --details")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains_all!(
            "Name: param3",
            "Value: {{ param1 }}",
            format!("Source: {env1}"),
            "Evaluated: false"
        ));
    cloudtruth!("param get param3 --details")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success()
        .stdout(contains_all!(
            "Name: param3",
            "Value: your-value",
            format!("Source: {env2}"),
            "Evaluated: true",
            "Raw: {{ param2 }}"
        ));

    cloudtruth!("param set param3 --evaluate false")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success();
    cloudtruth!("param list -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains_all!(
            format!("param1,first value,{env1},string,0,internal,false"),
            format!("param2,my-value,{env1},string,0,internal-evaluated,false"),
            format!("param3,{{{{ param1 }}}},{env1},string,0,internal,false"),
            format!("param4,{{{{ param1 }}}},{env1},string,0,internal-evaluated,false"),
        ));
    cloudtruth!("param list -f csv")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env2.name())
        .assert()
        .success()
        .stdout(contains_all!(
            format!("param1,other value,{env2},string,0,internal,false"),
            format!("param2,your-value,{env2},string,0,internal,false"),
            format!("param3,{{{{ param2 }}}},{env2},string,0,internal,false"),
            format!("param4,-,,string,0,internal,false"),
        ));
    cloudtruth!("param set param3 --value '{{{{ cloudtruth.environment }}}}' --evaluate true")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success();
    cloudtruth!("param list -v -f csv --evaluated")
        .envs(&envs)
        .env(CT_ENVIRONMENT, env1.name())
        .assert()
        .success()
        .stdout(contains_all!(
            "param2,my-value,my-value",
            format!("param3,{env1},{{{{ cloudtruth.environment }}}}")
        ));
}

#[test]
#[use_harness]
fn test_parameters_as_of_time() {
    let proj = Project::with_prefix("param-times").create();
    let env_a = Environment::with_prefix("env-a-param-times").create();
    let env_b = Environment::with_prefix("env-b-param-times").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name()
    };
    /* Create parameter in two environments */
    cloudtruth!("--env {env_a} param set some_param --value 'value a - first'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("--env {env_b} param set some_param --value 'value B1'")
        .envs(&envs)
        .assert()
        .success();
    let param_a1 = cloudtruth!("--env {env_a} param list --format json --show-times")
        .envs(&envs)
        .assert()
        .success()
        .get_param("some_param")
        .unwrap();
    assert_eq!(param_a1.value, "value a - first");
    let param_b1 = cloudtruth!("--env {env_b} param list --format json --show-times")
        .envs(&envs)
        .assert()
        .success()
        .get_param("some_param")
        .unwrap();
    assert_eq!(param_b1.value, "value B1");
    let modified_at_a1 = param_a1.modified_at.as_deref().unwrap();
    let modified_at_b1 = param_b1.modified_at.as_deref().unwrap();
    /* Update the parameters */
    cloudtruth!("--env {env_a} param set some_param --value 'value b - second'")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("--env {env_b} param set some_param --value 'value B2'")
        .envs(&envs)
        .assert()
        .success();
    let param_a2 = cloudtruth!("--env {env_a} param list --format json --show-times")
        .envs(&envs)
        .assert()
        .success()
        .get_param("some_param")
        .unwrap();
    assert_eq!(param_a2.value, "value b - second");
    let param_b2 = cloudtruth!("--env {env_b} param list --format json --show-times")
        .envs(&envs)
        .assert()
        .success()
        .get_param("some_param")
        .unwrap();
    assert_eq!(param_b2.value, "value B2");
    let modified_at_a2 = param_a2.modified_at.as_deref().unwrap();
    let modified_at_b2 = param_b2.modified_at.as_deref().unwrap();
    /* Verify `param get` command works with --as-of */
    cloudtruth!("--env {env_a} param get some_param --as-of '{modified_at_a1}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("value a - first"));
    cloudtruth!("--env {env_b} param get some_param --as-of '{modified_at_b1}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("value B1"));
    cloudtruth!("--env {env_a} param get some_param --as-of '{modified_at_a2}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("value b - second"));
    cloudtruth!("--env {env_b} param get some_param --as-of '{modified_at_b2}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("value B2"));
    cloudtruth!("--env {env_a} param get some_param -d")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            format!(
                "Created At: {created_at}",
                created_at = param_a2.created_at.as_deref().unwrap()
            ),
            format!("Modified At: {modified_at_a2}"),
            format!("Value: value b - second")
        ));
    cloudtruth!("--env {env_b} param get some_param --details --as-of '{modified_at_b1}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            format!(
                "Created At: {created_at}",
                created_at = param_b1.created_at.as_deref().unwrap()
            ),
            format!("Modified At: {modified_at_b1}"),
            format!("Value: value B1")
        ));
    /* Verify `param list` command works with --as-of */
    assert_eq!(
        param_a2,
        cloudtruth!("--env {env_a} param list --format json --show-times")
            .envs(&envs)
            .assert()
            .success()
            .get_param("some_param")
            .unwrap()
    );
    assert_eq!(
        param_b2,
        cloudtruth!("--env {env_b} param list --format json --show-times")
            .envs(&envs)
            .assert()
            .success()
            .get_param("some_param")
            .unwrap()
    );
    assert_eq!(
        param_a1,
        cloudtruth!(
            "--env {env_a} param list --format json --show-times --as-of '{modified_at_a1}'"
        )
        .envs(&envs)
        .assert()
        .success()
        .get_param("some_param")
        .unwrap()
    );
    assert_eq!(
        param_b1,
        cloudtruth!(
            "--env {env_b} param list --format json --show-times --as-of '{modified_at_b1}'"
        )
        .envs(&envs)
        .assert()
        .success()
        .get_param("some_param")
        .unwrap()
    );
    /* Verify `param environments` command with --as-of */
    let entries = cloudtruth!("param env some_param --show-times --format json")
        .envs(&envs)
        .assert()
        .success()
        .parse_param_env();
    let entry_a = entries.find_by_env(env_a.name()).unwrap();
    assert_eq!(entry_a.value, param_a2.value);
    assert_eq!(entry_a.created_at, param_a2.created_at);
    assert_eq!(entry_a.modified_at, param_a2.modified_at);
    let entry_b = entries.find_by_env(env_b.name()).unwrap();
    assert_eq!(entry_b.value, param_b2.value);
    assert_eq!(entry_b.created_at, param_b2.created_at);
    assert_eq!(entry_b.modified_at, param_b2.modified_at);
    let entries =
        cloudtruth!("param env some_param --show-times --format json --as-of '{modified_at_b1}'")
            .envs(&envs)
            .assert()
            .success()
            .parse_param_env();
    let entry_a = entries.find_by_env(env_a.name()).unwrap();
    assert_eq!(entry_a.value, param_a1.value);
    assert_eq!(entry_a.created_at, param_a1.created_at);
    assert_eq!(entry_a.modified_at, param_a1.modified_at);
    let entry_b = entries.find_by_env(env_b.name()).unwrap();
    assert_eq!(entry_b.value, param_b1.value);
    assert_eq!(entry_b.created_at, param_b1.created_at);
    assert_eq!(entry_b.modified_at, param_b1.modified_at);
    /* Verify `param export docker` with --as-of */
    cloudtruth!("--env {env_a} param export docker")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("SOME_PARAM=value b - second"));
    cloudtruth!("--env {env_a} param export docker --as-of '{modified_at_a1}'")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("SOME_PARAM=value a - first"));
    /* Error cases */
    cloudtruth!("param ls -v --as-of 3/24/2021")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param diff --as-of 3/24/2021")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param env some_param --as-of 3/24/2021")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
    cloudtruth!("param export shell --as-of 3/24/2021")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No EnvironmentLedger matches the given query"));
    cloudtruth!("param get some_param --as-of 3/24/2021")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
}

#[test]
#[use_harness]
fn test_parameters_rules_integer() {
    let proj = Project::with_prefix("integer-rules").create();
    let env = Environment::with_prefix("integer-rules").create();
    let envs = hashmap! {
        CT_PROJECT => proj.name(),
        CT_ENVIRONMENT => env.name()
    };
    /* Create a basic parameter without a value, so the rule cannot be violated */
    cloudtruth!("param set param1 --value 2154 --type integer")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param unset param1")
        .envs(&envs)
        .assert()
        .success();
    /* See no rules */
    cloudtruth!("param list --rules -vf csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("No parameter rules found in project"));
    /* Create parameter with constraints */
    cloudtruth!("param set param1 --min 1000 --max 3000")
        .envs(&envs)
        .assert()
        .success();
    /* See the 2 rules are registered */
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("param1,-,,integer,2,internal,false"));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            "param1,integer,max,3000",
            "param1,integer,min,1000"
        ));
    cloudtruth!("param list --rules")
        .envs(&envs)
        .assert()
        .success()
        .stdout(diff("param1\n"));
    /* test min and max */
    cloudtruth!("param set param1 -v 999")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Value is less than the minimum value of 1000"));
    cloudtruth!("param set param1 -v 3001")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Value is greater than the maximum value of 3000"));
    /* test middle value */
    cloudtruth!("param set param1 -v 2000")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("param1,2000,{env},integer,2,internal,false"));
    /* Update the rules */
    cloudtruth!("param set param1 --min 500")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --max 6000")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("param1,2000,{env},integer,2,internal,false"));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains_all!(
            "param1,integer,max,6000",
            "param1,integer,min,500"
        ));
    /* Remove the rules */
    cloudtruth!("param set param1 --no-max")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("param1,2000,{env},integer,1,internal,false"));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("param1,integer,min,500"));
    cloudtruth!("param set param1 --no-min")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("param1,2000,{env},integer,0,internal,false"));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("No parameter rules found in project"));
    /* Failed create rules while values in place */
    cloudtruth!("param set param1 --min 2002")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule create error: Rule may not be applied to param1",
            "Value is less than the minimum value"
        ));
    cloudtruth!("param set param1 --max 1998")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule create error: Rule may not be applied to param1",
            "Value is greater than the maximum value"
        ));
    cloudtruth!("param set param1 --min 1990")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --min 2003")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule update error: Rule may not be applied to param1",
            "Value is less than the minimum value"
        ));
    cloudtruth!("param set param1 --max 2010")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --max 1998")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule update error: Rule may not be applied to param1",
            "Value is greater than the maximum value"
        ));
    /* Invalid rules */
    cloudtruth!("param set param1 --max 1989")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Rule update error: Maximum constraint is less than an existing rule's minimum constraint"));
    cloudtruth!("param set param1 --min 2011")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("Rule update error: Minimum constraint is greater than an existing rule's maximum constraint"));
    /* Delete the rules */
    cloudtruth!("param set param1 --no-min")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --no-max")
        .envs(&envs)
        .assert()
        .success();
    /* Error cases */
    cloudtruth!("param set param1 --max-len -10 --min-len -1 --regex 'abc.*'")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains_all!(
            "max-len rules not valid for integer parameters",
            "min-len rules not valid for integer parameters",
            "regex rules not valid for integer parameters"
        ));
    cloudtruth!("param set param1 --min-len 10")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("min-len rules not valid for integer parameters"));
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("param1,2000,{env},integer,0,internal,false"));
    cloudtruth!("param ls --rules -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains("No parameter rules found in project"));
    /* See we don't leave any parameter behind when creating a parameter with an invalid rule */
    cloudtruth!("param del -y param1")
        .envs(&envs)
        .assert()
        .success();
    cloudtruth!("param set param1 --type integer --value 9 --max-len 100")
        .envs(&envs)
        .assert()
        .failure()
        .stderr(contains("max-len rules not valid for integer parameters"));
    cloudtruth!("param ls -v -f csv")
        .envs(&envs)
        .assert()
        .success()
        .stdout(contains!("No parameters found in project {proj}"));
}

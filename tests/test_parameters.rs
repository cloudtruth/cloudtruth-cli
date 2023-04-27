use cloudtruth_config::{CT_ENVIRONMENT, CT_PROJECT};
use integration_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_parameter_basic_empty_list() {
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
fn test_parameter_basic() {
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
fn test_parameter_basic_delete() {
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
fn test_parameter_basic_no_update() {
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
fn test_parameter_basic_no_value() {
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
fn test_parameter_basic_conflicting_options() {
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
fn test_parameter_basic_secret_list() {
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
fn test_parameter_basic_secret_idempotent() {
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
fn test_parameter_basic_secret_update() {
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
fn test_parameter_basic_secret_delete() {
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
fn test_parameter_basic_secret_no_value() {
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
fn test_parameter_export() -> Result<()> {
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
fn test_parameter_generate() -> Result<()> {
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
fn test_parameter_integration_errors() -> Result<()> {
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
fn test_parameter_names() -> Result<()> {
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
fn test_parameter_over_specified() -> Result<()> {
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
fn test_parameter_rules_bool() -> Result<()> {
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
fn test_parameter_rules_string() -> Result<()> {
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
fn test_parameter_secret_switch() -> Result<()> {
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
fn test_parameter_table_formats() -> Result<()> {
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
fn test_parameter_types() -> Result<()> {
    let proj = Project::with_prefix("param-types").create();
    trycmd::TestCases::new()
        .case("tests/snapshot-tests/parameters/parameter_types.md")
        .register_bin("cloudtruth", cli_bin_path!())
        .env("NO_COLOR", "1")
        .env(CT_PROJECT, proj.to_name())
        .insert_var("[PROJECT]", proj.to_name())?;
    Ok(())
}

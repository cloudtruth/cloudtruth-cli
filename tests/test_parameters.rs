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
        .stdout(contains_all([
            "Name: my_param",
            "Value: cRaZy value",
            "Source: default",
            "Secret: false",
            "Description: this is just a test description",
        ]));

    // idempotent
    cloudtruth!("--project {proj} parameters set my_param --value 'cRaZy value'")
        .assert()
        .success();
    cloudtruth!("--project {proj} parameters ls -v")
        .assert()
        .success()
        .stdout(contains_all([
            "my_param",
            "cRaZy value",
            "this is just a test description",
        ]));
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
        .stdout(contains_all([
            "my_param",
            "new_value",
            "this is just a test description",
        ]));
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
        .stdout(contains_all(["my_param", "my_value", "test description"]));
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
        .stdout(contains_all([
            "renamed_param",
            "value",
            "parameter rename test",
        ]));
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

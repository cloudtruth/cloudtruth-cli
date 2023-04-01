use integration_test_harness::prelude::*;
use std::path::Path;

#[test]
fn project_basic_examples() -> Result<()> {
    let name = Name::uuid();
    println!("{}", name);
    let old_name = name.clone();
    let new_name = Name::uuid_with_prefix("proj-rename");
    trycmd::TestCases::new()
        .register_bin("cloudtruth", Path::new(cli_bin_path!()))
        .case("examples/basic-usage/projects.md")
        .insert_var("[NAME]", name)?
        .insert_var("[OLD]", old_name)?
        .insert_var("[NEW]", new_name)?
        .run();
    Ok(())
}

#[integration_test]
fn project_basic() {
    let mut proj_name = Name::uuid_with_prefix("proj-name");

    // verify proj_name does not yet exist
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(contains(&proj_name).not());

    // create with a description
    cloudtruth!(
        "projects set {name} --desc 'Description on create'",
        name = proj_name
    )
    .assert()
    .success();

    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj_name},,Description on create")));

    // update the description
    cloudtruth!(
        "projects set {name} --desc 'Updated description'",
        name = proj_name
    )
    .assert()
    .success();
    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj_name},,Updated description")));

    // idempotent - do it again
    cloudtruth!(
        "projects set {name} --desc 'Updated description'",
        name = proj_name
    )
    .assert()
    .success();

    // rename
    let proj_rename = Name::uuid();
    cloudtruth!(
        "projects set {name1} --rename {name2}",
        name1 = proj_name,
        name2 = proj_rename
    )
    .assert()
    .success()
    .stdout(contains(format!("Updated project '{proj_rename}'")));

    proj_name = proj_rename;

    // nothing to update
    cloudtruth!("projects set {name}", name = proj_name)
        .assert()
        .success()
        .stderr(contains(format!(
            "Project '{proj_name}' not updated: no updated parameters provided"
        )));

    // test the list without the values
    cloudtruth!("projects list")
        .assert()
        .success()
        .stdout(contains(&proj_name).and(contains("Updated description").not()));

    // shows create/modified times
    cloudtruth!("projects list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(&proj_name))
                .and(contains("Updated description")),
        );

    // delete
    cloudtruth!("projects delete {name} --confirm", name = proj_name)
        .assert()
        .success();
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(contains(&proj_name).not());

    // do it again, see we have success and a warning
    cloudtruth!("projects delete {name} --confirm", name = proj_name)
        .assert()
        .success()
        .stderr(contains(format!("Project '{proj_name}' does not exist")));
}

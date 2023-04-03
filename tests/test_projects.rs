use integration_test_harness::prelude::*;

#[integration_test]
fn project_basic() {
    let mut proj_name = Name::uuid_with_prefix("proj-name");

    // verify proj_name does not yet exist
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(&proj_name)));

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
        .stdout(contains(&proj_name).and(not(contains("Updated description"))));

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
        .stdout(not(contains(&proj_name)));

    // do it again, see we have success and a warning
    cloudtruth!("projects delete {name} --confirm", name = proj_name)
        .assert()
        .success()
        .stderr(contains(format!("Project '{proj_name}' does not exist")));
}

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

#[integration_test]
fn project_parents() {
    let proj1 = ProjectName::uuid_with_prefix("proj-par-1").scoped();
    let proj2 = ProjectName::uuid_with_prefix("proj-mid-1").scoped();
    let proj3 = ProjectName::uuid_with_prefix("proj-chld-3").scoped();
    let proj4 = ProjectName::uuid_with_prefix("proj-chld-4").scoped();

    cloudtruth!("proj ls -v -f csv").assert().success().stdout(
        contains(format!("{proj1},,"))
            .and(contains(format!("{proj2},{proj1},")))
            .and(contains(format!("{proj3},{proj2},")))
            .and(contains(format!("{proj4},{proj2},"))),
    );

    cloudtruth!("proj tree")
        .assert()
        .success()
        .stdout(contains(format!(
            "{proj1}\n  {proj2}\n    {proj3}\n    {proj4}"
        )));

    cloudtruth!("proj delete {proj2} --confirm", proj2 = proj2)
        .assert()
        .failure()
        .stderr(
            contains(format!(
                "Cannot delete {proj2} because the following projects depend on it"
            ))
            .and(contains(&proj3).and(contains(&proj4))),
        );

    let proj5 = ProjectName::uuid_with_prefix("proj-par-5");
    let proj6 = ProjectName::uuid_with_prefix("proj-par-6");
    cloudtruth!(
        "proj set '{proj5}' --parent '{proj6}",
        proj5 = proj5,
        proj6 = proj6
    )
    .assert()
    .failure()
    .stderr(contains(format!("No parent project '{proj6}' found")));
    cloudtruth!(
        "proj set '{proj4}' --parent '{proj1}",
        proj4 = proj4,
        proj1 = proj1
    )
    .assert()
    .success()
    .stdout(contains(format!("Updated parent '{proj4}'")));
    cloudtruth!("proj ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj4},{proj1},")));

    cloudtruth!(
        "proj ls '{proj4}' --parent '{proj2} --desc 'My news description'",
        proj4 = proj4,
        proj2 = proj2
    )
    .assert()
    .success();
    cloudtruth!("proj ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj4},{proj1},My new description")));
}

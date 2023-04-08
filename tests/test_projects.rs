use integration_test_harness::prelude::*;

#[integration_test]
fn project_basic() {
    let mut proj = Project::uuid_with_prefix("proj-name");

    // verify proj_name does not yet exist
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(&proj)));

    // create with a description
    cloudtruth!(
        "projects set {name} --desc 'Description on create'",
        name = proj
    )
    .assert()
    .success();

    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj},,Description on create")));

    // update the description
    cloudtruth!(
        "projects set {name} --desc 'Updated description'",
        name = proj
    )
    .assert()
    .success();
    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj},,Updated description")));

    // idempotent - do it again
    cloudtruth!(
        "projects set {name} --desc 'Updated description'",
        name = proj
    )
    .assert()
    .success();

    // rename
    let proj_rename = Project::uuid_with_prefix("proj-rename");
    cloudtruth!(
        "projects set {name1} --rename {name2}",
        name1 = proj,
        name2 = proj_rename
    )
    .assert()
    .success()
    .stdout(contains(format!("Updated project '{proj_rename}'")));

    proj = proj_rename;

    // nothing to update
    cloudtruth!("projects set {name}", name = proj)
        .assert()
        .success()
        .stderr(contains(format!(
            "Project '{proj}' not updated: no updated parameters provided"
        )));

    // test the list without the values
    cloudtruth!("projects list")
        .assert()
        .success()
        .stdout(contains(&proj).and(not(contains("Updated description"))));

    // shows create/modified times
    cloudtruth!("projects list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(&proj))
                .and(contains("Updated description")),
        );

    // delete
    cloudtruth!("projects delete {name} --confirm", name = proj)
        .assert()
        .success();
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(&proj)));

    // do it again, see we have success and a warning
    cloudtruth!("projects delete {name} --confirm", name = proj)
        .assert()
        .success()
        .stderr(contains(format!("Project '{proj}' does not exist")));
}

#[integration_test]
fn project_parents() {
    let proj1 = ScopedProject::uuid_with_prefix("proj-par-1");
    let proj2 = ProjectBuilder::uuid_with_prefix("proj-mid-1")
        .parent(&proj1)
        .build_scoped();
    let proj3 = ProjectBuilder::uuid_with_prefix("proj-chld-3")
        .parent(&proj2)
        .build_scoped();
    let proj4 = ProjectBuilder::uuid_with_prefix("proj-chld-4")
        .parent(&proj2)
        .build_scoped();

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

    let proj5 = Project::uuid_with_prefix("proj-par-5");
    let proj6 = Project::uuid_with_prefix("proj-par-6");
    cloudtruth!(
        "proj set '{proj5}' --parent '{proj6}'",
        proj5 = proj5,
        proj6 = proj6
    )
    .assert()
    .failure()
    .stderr(contains(format!("No parent project '{proj6}' found")));
    cloudtruth!(
        "proj set '{proj4}' --parent '{proj1}'",
        proj4 = proj4,
        proj1 = proj1
    )
    .assert()
    .success()
    .stdout(contains(format!("Updated project '{proj4}'")));
    cloudtruth!("proj ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj4},{proj1},")));

    cloudtruth!(
        "proj set '{proj4}' --parent '{proj2}' --desc 'My new description'",
        proj4 = proj4,
        proj2 = proj2
    )
    .assert()
    .success();
    cloudtruth!("proj ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{proj4},{proj2},My new description")));
}

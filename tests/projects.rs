use integration_test_harness::prelude::*;

const TEST_PAGE_SIZE: usize = 5;

#[integration_test]
fn test_projects_basic() {
    let mut proj = Project::with_prefix("proj-name");

    // verify proj_name does not yet exist
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(&proj)));

    // create with a description
    cloudtruth!("projects set {proj} --desc 'Description on create'")
        .assert()
        .success();

    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{proj},,Description on create"));

    // update the description
    cloudtruth!("projects set {proj} --desc 'Updated description'")
        .assert()
        .success();

    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{proj},,Updated description"));

    // idempotent - do it again
    cloudtruth!("projects set {proj} --desc 'Updated description'")
        .assert()
        .success();

    // rename
    let proj_rename = Project::with_prefix("proj-rename");
    cloudtruth!("projects set {proj} --rename {proj_rename}")
        .assert()
        .success()
        .stdout(contains!("Updated project '{proj_rename}'"));

    proj = proj_rename;

    // nothing to update
    cloudtruth!("projects set {proj}")
        .assert()
        .success()
        .stderr(contains!(
            "Project '{proj}' not updated: no updated parameters provided"
        ));

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
    cloudtruth!("projects delete {proj} --confirm")
        .assert()
        .success();
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(&proj)));

    // do it again, see we have success and a warning
    cloudtruth!("projects delete {proj} --confirm")
        .assert()
        .success()
        .stderr(contains!("Project '{proj}' does not exist"));
}

#[integration_test]
fn test_projects_parents() {
    let proj1 = ScopedProject::with_prefix("proj-par-1");
    let proj2 = ProjectBuilder::with_prefix("proj-mid-1")
        .parent(&proj1)
        .build_scoped();
    let proj3 = ProjectBuilder::with_prefix("proj-chld-3")
        .parent(&proj2)
        .build_scoped();
    let proj4 = ProjectBuilder::with_prefix("proj-chld-4")
        .parent(&proj2)
        .build_scoped();

    cloudtruth!("proj ls -v -f csv").assert().success().stdout(
        contains!("{proj1},,")
            .and(contains!("{proj2},{proj1},"))
            .and(contains!("{proj3},{proj2},"))
            .and(contains!("{proj4},{proj2},")),
    );

    cloudtruth!("proj tree")
        .assert()
        .success()
        .stdout(contains!("{proj1}\n  {proj2}\n    {proj3}\n    {proj4}"));

    cloudtruth!("proj delete {proj2} --confirm")
        .assert()
        .failure()
        .stderr(
            contains!("Cannot delete {proj2} because the following projects depend on it")
                .and(contains(&proj3).and(contains(&proj4))),
        );

    let proj5 = Project::with_prefix("proj-par-5");
    let proj6 = Project::with_prefix("proj-par-6");
    cloudtruth!("proj set '{proj5}' --parent '{proj6}'")
        .assert()
        .failure()
        .stderr(contains!("No parent project '{proj6}' found"));
    cloudtruth!("proj set '{proj4}' --parent '{proj1}'")
        .assert()
        .success()
        .stdout(contains!("Updated project '{proj4}'"));
    cloudtruth!("proj ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{proj4},{proj1},"));

    cloudtruth!("proj set '{proj4}' --parent '{proj2}' --desc 'My new description'")
        .assert()
        .success();
    cloudtruth!("proj ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{proj4},{proj2},My new description"));
}

#[integration_test]
fn test_projects_pagination() {
    let page_size = TEST_PAGE_SIZE;
    // we store the project names so they're not instantly dropped and deleted
    let _projects: Vec<ScopedProject> = (0..=page_size)
        .map(|n| ScopedProject::with_prefix(format!("proj-page-{}", n)))
        .collect();
    cloudtruth!("proj ls")
        .rest_debug()
        .page_size(page_size)
        .assert()
        .success()
        .paginated(page_size);
}

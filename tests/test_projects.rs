use cloudtruth_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_projects_basic() {
    // Initialize project data but do not create yet
    let proj = Project::with_prefix("proj-name").description("Description on create");

    // verify proj_name does not yet exist
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(proj.name())));

    let mut proj = proj.create();

    // create/delete the project within scope of this closure
    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{proj},,,Description on create"));
    // update the description
    cloudtruth!("projects set {proj} --desc 'Updated description'")
        .assert()
        .success();
    cloudtruth!("projects ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{proj},,,Updated description"));
    // idempotent - do it again
    cloudtruth!("projects set {proj} --desc 'Updated description'")
        .assert()
        .success();
    // rename the project
    proj.rename(Name::with_prefix("proj-rename"));
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
        .stdout(contains(proj.name()).and(not(contains("Updated description"))));
    // shows create/modified times
    cloudtruth!("projects list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(proj.name()))
                .and(contains("Updated description")),
        );

    // explicitly delete
    let deleted_proj = Project::from_name(proj.name().clone());
    drop(proj);

    // verify deletion
    cloudtruth!("projects ls -v")
        .assert()
        .success()
        .stdout(not(contains(deleted_proj.name())));

    // try to delete again, see we have success and a warning
    cloudtruth!("projects delete {deleted_proj} --confirm")
        .assert()
        .success()
        .stderr(contains!("Project '{deleted_proj}' does not exist"));
}

#[test]
#[use_harness]
fn test_projects_parents() {
    let proj1 = Project::with_prefix("proj-par-1").create();
    let proj2 = Project::with_prefix("proj-mid-1").parent(&proj1).create();
    let proj3 = Project::with_prefix("proj-chld-3").parent(&proj2).create();
    let proj4 = Project::with_prefix("proj-chld-4").parent(&proj2).create();

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
                .and(contains(proj3.name()).and(contains(proj4.name()))),
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
        .stdout(contains_all!(
            format!("{proj4}"),
            format!("{proj2}"),
            "My new description"
        ));
}

#[test]
#[use_harness]
fn test_projects_pagination() {
    const PAGE_SIZE: usize = 5;
    // we store the project names so they're not instantly dropped and deleted
    let _projects: Vec<Scope<Project>> = (0..=PAGE_SIZE)
        .map(|n| Project::with_prefix(format!("proj-page-{}", n)).create())
        .collect();
    cloudtruth!("proj ls")
        .rest_debug()
        .page_size(PAGE_SIZE)
        .assert()
        .success()
        .paginated(PAGE_SIZE);
}

#[test]
#[use_harness]
fn test_projects_copy() {
    let proj = Project::with_prefix("proj-copy-src").create();
    let proj2 = proj.copy(Name::with_prefix("proj-copy-dest"));
    cloudtruth!("proj ls")
        .assert()
        .success()
        .stdout(contains(proj.name()).and(contains(proj2.name())));
}

use cloudtruth_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_group_basic() {
    let group = Group::with_prefix("group-name").description("Description on create");
    // make sure group doesn't exist
    cloudtruth!("groups ls -v -f csv")
        .assert()
        .success()
        .stdout(not(contains(group.name())));
    cloudtruth!("groups get {group}")
        .assert()
        .failure()
        .stderr(contains!("The group '{group}' could not be found"));
    let group = group.create();
    cloudtruth!("groups list --values --format csv")
        .assert()
        .success()
        .stdout(contains!("{group},Description on create,"));
    // update description
    cloudtruth!("groups set {group} --desc 'Updated description'")
        .assert()
        .success();
    cloudtruth!("groups ls -v -f json")
        .assert()
        .success()
        .stdout(json(prop(
            "groups",
            find_entry(
                prop("Name", value(group.name())),
                prop("Description", value("Updated description")),
            ),
        )));
    // idempotent
    cloudtruth!("groups set {group} --desc 'Updated description'")
        .assert()
        .success();
    cloudtruth!("groups get {group}")
        .assert()
        .success()
        .stdout(contains!("Name: {group}").and(contains_all!(
            "Description: Updated description",
            "Created At: ",
            "Modified At: ",
        )));
    // show modification times
    cloudtruth!("groups list --show-times -f csv")
        .assert()
        .success()
        .stdout(contains_all!(
            "Created At,Modified At",
            group.name().as_str(),
            "Updated description",
        ));
    //explicitly delete
    let deleted_group = Group::from_name(group.name().clone());
    drop(group);
    // try to delete again
    cloudtruth!("groups delete {deleted_group} --confirm")
        .assert()
        .success()
        .stderr(contains!("Group '{deleted_group}' does not exist"));
}

#[test]
#[use_harness]
fn test_group_basic_rename() {
    let mut group = Group::with_prefix("group-name")
        .description("Description on create")
        .create();
    let orig_group = group.clone();
    group.rename(Name::with_prefix("group-rename"));
    cloudtruth!("groups list")
        .assert()
        .success()
        .stdout(contains(group.name()).and(not(contains(orig_group.name()))));
}

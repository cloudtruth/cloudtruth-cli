use integration_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_type_basic() {
    let param_type = ParamType::with_prefix("type-name").description("Description on create");
    cloudtruth!("types ls -v")
        .assert()
        .success()
        .stdout(not(contains(param_type.name())));
    // create
    let mut param_type = param_type.create();
    cloudtruth!("types ls -f json")
        .assert()
        .success()
        .stdout(json(prop(
            "parameter-type",
            find_entry(
                prop("Name", value(param_type.name())),
                prop("Parent", value("string"))
                    .and(prop("Description", value("Description on create"))),
            ),
        )));
    // update description
    cloudtruth!("types set {param_type} --desc 'Updated description'")
        .assert()
        .success();
    cloudtruth!("types ls -f csv")
        .assert()
        .success()
        .stdout(contains!("{param_type},string,0,Updated description"));
    // idempotent
    cloudtruth!("types set {param_type} --desc 'Updated description'")
        .assert()
        .success();
    param_type.rename(Name::with_prefix("type-rename"));
    cloudtruth!("types list")
        .assert()
        .success()
        .stdout(contains(param_type.name()).and(not(contains("Updated description"))));
    cloudtruth!("types list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(param_type.name()).and(contains("Updated description"))),
        );
    //delete
    let deleted_param_type = ParamType::from_name(param_type.name().clone());
    drop(param_type);
    // try to delete again
    cloudtruth!("types delete {deleted_param_type} --confirm")
        .assert()
        .success();
}

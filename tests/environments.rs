use integration_test_harness::prelude::*;

#[integration_test]
fn test_environments_basic() {
    let mut env = Environment::uuid_with_prefix("env-name");

    // verify env_name does not yet exist
    cloudtruth!("environments ls -v")
        .assert()
        .success()
        .stdout(not(contains(&env)));

    // create with a description
    cloudtruth!("environments set {env} --desc 'Description on create'")
        .assert()
        .success();

    cloudtruth!("environments ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{env},default,Description on create")));

    // update the description
    cloudtruth!("environments set {env} --desc 'Updated description'")
        .assert()
        .success();

    cloudtruth!("environments ls -v -f csv")
        .assert()
        .success()
        .stdout(contains(format!("{env},default,Updated description")));

    // idempotent - do it again
    cloudtruth!("environments set {env} --desc 'Updated description'")
        .assert()
        .success();

    // rename
    let env_rename = Environment::uuid_with_prefix("env-rename");
    cloudtruth!("environments set {env} --rename {env_rename}")
        .assert()
        .success()
        .stdout(contains(format!("Updated environment '{env_rename}'")));

    env = env_rename;

    // nothing to update
    cloudtruth!("environments set {env}")
        .assert()
        .success()
        .stderr(contains(format!(
            "Environment '{env}' not updated: no updated parameters provided"
        )));

    // test the list without the values
    cloudtruth!("environments list")
        .assert()
        .success()
        .stdout(contains(&env).and(not(contains("Updated description"))));

    // shows create/modified times
    cloudtruth!("environments list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(&env))
                .and(contains("Updated description")),
        );

    // delete
    cloudtruth!("environments delete {env} --confirm")
        .assert()
        .success();
    cloudtruth!("environments ls -v")
        .assert()
        .success()
        .stdout(not(contains(&env)));

    // do it again, see we have success and a warning
    cloudtruth!("environments delete {env} --confirm")
        .assert()
        .success()
        .stderr(contains(format!("Environment '{env}' does not exist")));
}

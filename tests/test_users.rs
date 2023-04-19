use integration_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_user_basic() {
    let user = User::with_prefix("user-name").description("Description on create");

    cloudtruth!("users ls -v -f csv")
        .assert()
        .success()
        .stdout(not(contains(user.name())));

    cloudtruth!("users get {user}")
        .assert()
        .failure()
        .stdout(not(contains!("The user '{user}' could not be found")));

    user.clone().with_scope(|user| {
        cloudtruth!("users list --values --format csv")
            .assert()
            .success()
            .stdout(contains!("{user},service,viewer,,Description on create"));

        // update the description
        cloudtruth!("users set {user} --desc 'Updated description'")
            .assert()
            .success();

        cloudtruth!("users ls -v -f json")
            .assert()
            .success()
            .stdout(json(prop(
                "user",
                find_entry(
                    prop("Name", value(user.to_name())),
                    prop("Type", value("service"))
                        .and(prop("Role", value("viewer")))
                        .and(prop("Description", value("Updated description"))),
                ),
            )));

        // idempotent
        cloudtruth!("users set {user} --desc 'Updated description'")
            .assert()
            .success();

        // use the new API key
        let api_key = user.api_key();
        cloudtruth!("--api-key {api_key} env ls -vf csv")
            .assert()
            .success();

        // test if new API key is viewer role
        cloudtruth!("--api-key {api_key} user set {user} --role owner")
            .assert()
            .failure()
            .stderr(contains(
                "You do not have permission to perform this action",
            ));

        // update the user role
        cloudtruth!("users set {user} --role contrib")
            .assert()
            .success()
            .stdout(contains!("Updated user '{user}'"));

        cloudtruth!("users ls -v -f json")
            .assert()
            .success()
            .stdout(json(prop(
                "user",
                find_entry(
                    prop("Name", value(user.to_name())),
                    prop("Type", value("service"))
                        .and(prop("Role", value("contrib")))
                        .and(prop("Description", value("Updated description"))),
                ),
            )));

        cloudtruth!("users get {user}").assert().success().stdout(
            contains!("Name: {user}")
                .and(contains("Role: contrib"))
                .and(contains("Organization: "))
                .and(contains("Description: Updated description"))
                .and(contains("Type: service"))
                .and(contains("Created At: "))
                .and(contains("Modified At: "))
                .and(contains("Last Used At: ")),
        );

        // Nothing to update
        cloudtruth!("users set {user}")
            .assert()
            .success()
            .stderr(contains!(
                "User '{user}' not updated: no updated parameters provided"
            ));

        // use the API key after updating role
        let api_key = user.api_key();
        cloudtruth!("--api-key {api_key} env ls -vf csv")
            .assert()
            .success();

        // try creating a new owner
        let user2 = User::with_prefix("another-user");
        cloudtruth!("--api-key {api_key} user set {user2} --role owner")
            .assert()
            .failure()
            .stderr(contains(
                "You do not have permission to perform this action",
            ));

        // check whole line matches
        cloudtruth!("users list").assert().success().stdout(
            contains(user.name())
                .and(not(contains(user2.name())))
                .and(not(contains("Updated description"))),
        );

        cloudtruth!("--api-key {api_key} users current")
            .assert()
            .success()
            .stdout(contains!("Name: {user}").and(contains("Role: contrib")));

        cloudtruth!("users list --show-times -f csv")
            .assert()
            .success()
            .stdout(
                contains("Created At,Modified At,Last Used At")
                    .and(contains(user.name()))
                    .and(contains("Updated description")),
            );
    });
    // Try to delete again
    cloudtruth!("users delete {user} --confirm")
        .assert()
        .success()
        .stderr(contains!("User '{user}' does not exist"));
}

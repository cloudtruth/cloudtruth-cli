use cloudtruth_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_configuration_profile() {
    // create test user
    let test_user = User::with_prefix("configuration-user").create();

    // test missing profile
    let missing_profile = Profile::with_prefix("missing-profile");
    cloudtruth!("--profile '{missing_profile}' env ls -v")
        .assert()
        .failure()
        .stderr(contains!(
            "Failed to load configuration from profile '{missing_profile}'"
        ));

    // create profile with test user API key
    let test_profile = Profile::with_prefix("configuration-profile")
        .api_key(test_user.api_key())
        .create();

    // verify profile exists
    cloudtruth!("--profile '{test_profile}' configuration profile list")
        .assert()
        .success()
        .stdout(contains(test_profile.name()));

    // update some profile fields
    cloudtruth!("conf prof set '{test_profile}' -d 'Test Profile' -e 'config-test-environment'")
        .assert()
        .success()
        .stdout(contains!("Updated profile '{test_profile}'"));

    // make sure API key is not shown, but other parameters are
    cloudtruth!("conf prof ls -vf csv")
        .assert()
        .success()
        .stdout(contains!(
            "{test_profile},*****,config-test-environment,,Test Profile"
        ));

    // update some more fields
    cloudtruth!(
        "conf prof set '{test_profile}' -p 'config-test-project' -d 'Updated Test Profile'"
    )
    .assert()
    .success();

    // show all parameters including secrets
    cloudtruth!("conf prof ls -vsf csv")
        .assert()
        .success()
        .stdout(contains!(
        "{test_profile},{api_key},config-test-environment,config-test-project,Updated Test Profile",
        api_key = test_user.api_key()
    ));

    // update without any options produces warning
    cloudtruth!("conf prof set '{test_profile}'")
        .assert()
        .success()
        .stderr(contains!("Nothing to change for profile '{test_profile}'"));

    // test JSON
    cloudtruth!("conf prof list -vsf json")
        .assert()
        .success()
        .stdout(json(prop(
            "profile",
            find_entry(
                prop("Name", value(test_profile.name().as_str())),
                all!(
                    prop("API", value(test_user.api_key())),
                    prop("Environment", value("config-test-environment")),
                    prop("Project", value("config-test-project")),
                    prop("Description", value("Updated Test Profile"))
                ),
            ),
        )));

    // create child profile
    let child_proj = Project::with_prefix("config-test-child-project");
    let child_profile = Profile::with_prefix("config-child-profile")
        .source(&test_profile)
        .project(&child_proj)
        .create();

    // verify child was created
    cloudtruth!("conf prof ls -v -f csv")
        .assert()
        .success()
        .stdout(contains_all([child_profile.name(), child_proj.name()]));

    // try to create child with non-existence parent
    let invalid_child_profile = Profile::with_prefix("config-missing-parent");
    let missing_profile = Profile::with_prefix("missing-profile");
    cloudtruth!("conf prof set '{invalid_child_profile}' -s '{missing_profile}'")
        .assert()
        .failure()
        .stderr(contains!(
            "Source profile '{missing_profile}' does not exist"
        ));

    // delete profiles
    drop(child_profile);
    let deleted_profile = Profile::from_name(test_profile.name().clone());
    drop(test_profile);

    // verify it is gone
    cloudtruth!("conf prof ls")
        .assert()
        .success()
        .stdout(not(contains(deleted_profile.name())));

    // delete is idempotent
    cloudtruth!("config prof delete -y '{deleted_profile}'")
        .assert()
        .success()
        .stderr(contains!("Profile '{deleted_profile}' does not exist"));
}

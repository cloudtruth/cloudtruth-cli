use cloudtruth_config::{
    CT_PROFILE, CT_REQ_TIMEOUT, CT_REST_DEBUG, CT_REST_PAGE_SIZE, CT_REST_SUCCESS,
};
use cloudtruth_test_harness::output::profile::*;
use cloudtruth_test_harness::prelude::*;
use maplit::hashmap;

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

#[test]
#[use_harness]
fn test_configuration_current() {
    let cmd = cloudtruth!("config current").assert().success();
    let orig_profile = Profile::from_string(String::from_utf8_lossy(&cmd.get_output().stdout));
    let test_proj = Project::with_prefix("current-profile-proj");
    let test_env = Environment::with_prefix("current-profile-env");
    let test_profile = Profile::with_prefix("current-profile")
        .api_key("bogus-key-value")
        .project(&test_proj)
        .env(&test_env)
        .create();

    // current profile is unchanged
    cloudtruth!("config current")
        .assert()
        .success()
        .stdout(eq(orig_profile.name().as_str()));

    // create new environment map
    let env_map = hashmap! {
        CT_PROFILE => test_profile.name().as_str()
    };

    // check current profile with new environment
    cloudtruth!("config current")
        .env_clear()
        .envs(&env_map)
        .assert()
        .success()
        .stdout(ne(orig_profile.name().as_str()).and(contains(test_profile.name())));

    // Check all profile parameters

    let profile_json = cloudtruth!("config current -sf json")
        .env_clear()
        .envs(&env_map)
        .assert()
        .success()
        .parse_profile_parameters();

    let expected_param_names = vec![
        "Profile",
        "API key",
        "Organization",
        "User",
        "Role",
        "Project",
        "Environment",
    ];
    let actual_param_names = profile_json.param_names();
    assert_eq!(expected_param_names, actual_param_names);

    let param = profile_json.find_param("Profile");
    assert_eq!(param.value, test_profile.name().as_str());
    assert_eq!(param.source, "shell");

    let param = profile_json.find_param("API key");
    assert_eq!(param.value, "bogus-key-value");
    assert_eq!(param.source, format!("profile ({test_profile})"));

    let param = profile_json.find_param("Project");
    assert_eq!(param.value, test_proj.name().as_str());
    assert_eq!(param.source, format!("profile ({test_profile})"));

    let param = profile_json.find_param("Environment");
    assert_eq!(param.value, test_env.name().as_str());
    assert_eq!(param.source, format!("profile ({test_profile})"));

    let param = profile_json.find_param("Organization");
    assert_eq!(param.value, "");
    assert_eq!(param.value, "");

    let param = profile_json.find_param("User");
    assert_eq!(param.value, "");
    assert_eq!(param.value, "");

    let param = profile_json.find_param("Role");
    assert_eq!(param.value, "");
    assert_eq!(param.value, "");

    // Extended profile configuration
    let env_map = hashmap! {
        CT_PROFILE => test_profile.name().as_str(),
        CT_REST_SUCCESS => "a,b,c,d",
        CT_REST_DEBUG => "false",
        CT_REST_PAGE_SIZE => "9",
        CT_REQ_TIMEOUT => "33"
    };

    let profile_json = cloudtruth!("config current -xsf json")
        .env_clear()
        .envs(&env_map)
        .assert()
        .success()
        .parse_profile_parameters();

    let expected_param_names = vec![
        "Profile",
        "API key",
        "Organization",
        "User",
        "Role",
        "Project",
        "Environment",
        "CLI version",
        "Server URL",
        "Request timeout",
        "REST debug",
        "REST success",
        "REST page size",
        "Accept Invalid Certs",
    ];
    let actual_param_names = profile_json.param_names();
    assert_eq!(expected_param_names, actual_param_names);

    let param = profile_json.find_param("REST debug");
    assert_eq!(param.value, "false");
    assert_eq!(param.source, "shell");

    let param = profile_json.find_param("REST success");
    assert_eq!(param.value, "a, b, c, d");
    assert_eq!(param.source, "shell");

    let param = profile_json.find_param("REST page size");
    assert_eq!(param.value, "9");
    assert_eq!(param.source, "shell");

    let param = profile_json.find_param("Request timeout");
    assert_eq!(param.value, "33");
    assert_eq!(param.source, "shell");

    // test with command line arguments

    let profile_json = cloudtruth!(
        "--api-key 'bogus-key-value' --profile '{test_profile}' config current -sf json"
    )
    .env_clear()
    .envs(&env_map)
    .assert()
    .success()
    .parse_profile_parameters();

    let param = profile_json.find_param("Profile");
    assert_eq!(param.value, test_profile.name().as_str());
    assert_eq!(param.source, "argument");

    let param = profile_json.find_param("API key");
    assert_eq!(param.value, "bogus-key-value");
    assert_eq!(param.source, "argument");

    let param = profile_json.find_param("Organization");
    assert_eq!(param.value, "");
    assert_eq!(param.value, "");

    let param = profile_json.find_param("User");
    assert_eq!(param.value, "");
    assert_eq!(param.value, "");

    let param = profile_json.find_param("Role");
    assert_eq!(param.value, "");
    assert_eq!(param.value, "");

    let deleted_profile = Profile::from_name(test_profile.name().clone());
    drop(test_profile);

    // when profile is not found, command succeeds without the bits from the config
    cloudtruth!("config current -s")
        .env_clear()
        .env(CT_PROFILE, deleted_profile.name().as_str())
        .assert()
        .success()
        .stdout(
            contains(deleted_profile.name().as_str())
                .and(not(contains("bogus-key-value")))
                .and(not(contains(test_env.name().as_str())))
                .and(not(contains(test_proj.name().as_str())))
                .and(not(contains!("profile ({deleted_profile})"))),
        );

    // use default environment variables
    let profile_json = cloudtruth!("config current -f json")
        .assert()
        .success()
        .parse_profile_parameters();
    let param = profile_json.find_param("API key");
    assert_eq!("*****", param.value);
    let param = profile_json.find_param("Organization");
    assert_ne!("", param.value);
    assert_eq!("API key", param.source);
    let param = profile_json.find_param("User");
    assert_ne!("", param.value);
    assert_eq!("API key", param.source);
    let param = profile_json.find_param("Role");
    assert!(["owner", "admin", "contrib"].contains(&param.value.as_str()));
    assert_eq!("API key", param.source);
}

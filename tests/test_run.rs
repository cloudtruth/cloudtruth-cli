use cloudtruth_config::{CT_API_KEY, CT_PROFILE, CT_PROJECT};
use cloudtruth_test_harness::{output::profile::ParseProfileListExt, prelude::*};
use maplit::hashmap;

#[cfg(not(target_os = "windows"))]
const PRINTENV: &str = "printenv";
#[cfg(target_os = "windows")]
const PRINTENV: &str = "SET";

#[test]
#[use_harness]
fn test_run_inheritance_env_only() {
    let proj = Project::with_prefix("run-env-proj").create();

    cloudtruth!("--project '{proj}' run --inherit none -c '{PRINTENV}'")
        .env("SOME_PARAM_NAME", "env_value")
        .assert()
        .success()
        .stdout(not(contains("SOME_PARAM_NAME")));

    for inherit in ["underlay", "overlay", "Exclusive"] {
        cloudtruth!("--project '{proj}' run --inherit {inherit} -c '{PRINTENV}'")
            .env("SOME_PARAM_NAME", "env_value")
            .assert()
            .success()
            .stdout(contains("SOME_PARAM_NAME=env_value"));
    }
}

#[test]
#[use_harness]
fn test_run_inheritance_coordination() {
    let proj = Project::with_prefix("run-inherit-proj").create();

    let env_map = hashmap! {
        CT_PROJECT => proj.name().as_str(),
        "SOME_PARAM_NAME" => "env_value"
    };

    cloudtruth!("param set SOME_PARAM_NAME --value ct_value")
        .envs(&env_map)
        .assert()
        .success();

    cloudtruth!("--project '{proj}' run --inherit none -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(contains("SOME_PARAM_NAME=ct_value"));

    cloudtruth!("--project '{proj}' run --inherit underlay -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(contains("SOME_PARAM_NAME=env_value"));

    cloudtruth!("--project '{proj}' run --inherit overlay -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(contains("SOME_PARAM_NAME=ct_value"));

    //unspecified inherit is the same as overlay
    cloudtruth!("run -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(contains("SOME_PARAM_NAME=ct_value"));

    cloudtruth!("--project '{proj}' run --inherit exclusive -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .failure()
        .stderr(contains(
            "Conflicting definitions in run environment for: SOME_PARAM_NAME",
        ));
}

#[test]
#[use_harness]
fn test_run_permissive() {
    let proj = Project::with_prefix("run-permissive-proj").create();

    let mut env_map = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };

    let profiles;
    if std::env::var_os(CT_API_KEY).is_none() {
        profiles = cloudtruth!("config profile list -vsf json")
            .assert()
            .success()
            .parse_profile_list();
        let prof_name = std::env::var(CT_PROFILE).unwrap_or("default".into());
        let profile = profiles
            .find_by_name(&prof_name)
            .unwrap_or_else(|| panic!("Could not find profile named '{prof_name}'"));
        env_map.insert(CT_API_KEY, profile.api.as_str());
    }

    cloudtruth!("run -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(not(contains(CT_API_KEY)));

    cloudtruth!("run --permissive -- {PRINTENV}")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(contains(CT_API_KEY));
}

#[test]
#[use_harness]
fn test_run_arg_with_spaces() {
    let proj = Project::with_prefix("run-spaces").create();
    let test_file = TestFile::new().unwrap();

    cloudtruth!("--project '{proj}' run -i none -- '{PRINTENV} > {test_file}' {PRINTENV}")
        .assert() // NOTE: whether this cmd fails or not may depend on platform
        .stderr(contains("command contains spaces, and may fail"));
}

#[test]
#[use_harness]
fn test_run_time() {
    let proj = Project::with_prefix("proj-old-run").create();

    cloudtruth!("--project '{proj}' param set my_param --value first-value")
        .assert()
        .success();

    let cmd = cloudtruth!("--project '{proj}' param list -vf json --show-times")
        .assert()
        .success();
    let json = serde_json::from_slice::<serde_json::Value>(&cmd.get_output().stdout)
        .expect("Unable to parse params JSON");
    let param = json
        .as_object()
        .expect("Expected top-level JSON object")
        .get("parameter")
        .expect("No 'parameter' property found")
        .as_array()
        .expect("Expected parameters array")
        .iter()
        .find(|param| match param.get("Name") {
            Some(name) => name == "my_param",
            None => false,
        })
        .expect("Could not find paramer 'my_param' in list");

    let orig_time = param
        .get("Modified At")
        .expect("Unable to find 'Modified At' property in parameter JSON")
        .as_str()
        .expect("Expected 'Modified At' to be a string");

    cloudtruth!("--project '{proj}' param set my_param --value second-value")
        .assert()
        .success();
    cloudtruth!("--project '{proj}' run --as-of '{orig_time}' -- {PRINTENV}")
        .assert()
        .success()
        .stdout(contains("first-value").and(not(contains("second-value"))));
    cloudtruth!("--project '{proj}' run -- {PRINTENV}")
        .assert()
        .success()
        .stdout(not(contains("first-value")).and(contains("second-value")));
}

#[test]
#[use_harness]
fn test_run_strict() {
    let proj = Project::with_prefix("run-strict").create();
    // create param without value
    cloudtruth!("--project '{proj}' param set SOME_PARAM_NAME")
        .assert()
        .success();
    // assert failure when param has no value
    cloudtruth!("--project '{proj}' run --strict -- {PRINTENV}")
        .assert()
        .failure()
        .stderr(contains("parameter found without a value"));
    // give param a value
    cloudtruth!("--project '{proj}' param set SOME_PARAM_NAME --value some-value")
        .assert()
        .success();
    cloudtruth!("--project '{proj}' run --strict -- {PRINTENV}")
        .assert()
        .success()
        .stdout(contains("SOME_PARAM_NAME"));
}

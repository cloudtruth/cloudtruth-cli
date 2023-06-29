use cloudtruth_config::{CT_API_KEY, CT_ENVIRONMENT, CT_PROJECT};
use integration_test_harness::prelude::*;
use maplit::hashmap;

#[use_harness]
#[test]
fn test_audit_logs() {
    /* Create admin user for testing */
    let test_user = User::with_prefix("log-user").role("admin").create();
    /* Take a summary snapshot */
    let cmd = cloudtruth!("audit summary")
        .env(CT_API_KEY, test_user.api_key())
        .assert()
        .success();
    let _summary = cmd.get_output();
    /* Create test project */
    let proj = Project::with_prefix("audit-proj").create();
    /* Create test environment */
    let env = Environment::with_prefix("audit-env").create();

    /* Environment variables map */
    let env_map = hashmap! {
        CT_API_KEY => test_user.api_key(),
        CT_PROJECT => proj.name().as_str(),
        CT_ENVIRONMENT => env.name().as_str()
    };

    /* Create test parameter */
    cloudtruth!("param set audit-param -v 'this is the value for the audit log test'")
        .envs(&env_map)
        .assert()
        .success();

    /* Create test template */
    let temp_file = TestFile::with_contents("# this template has just fixed text").unwrap();
    cloudtruth!("template set my-audit-template -b '{temp_file}'")
        .envs(&env_map)
        .assert()
        .success();

    let cmd = cloudtruth!("audit-logs ls -f json -m 20 --project '{proj}'")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(4, entries.len());
    assert_eq!(4, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Project").count());
    assert_eq!(1, entries.find_by_type("Parameter").count());
    assert_eq!(1, entries.find_by_type("Template").count());
    assert_eq!(1, entries.find_by_type("Value").count());

    let cmd = cloudtruth!("audit-logs ls -f json -m 20 --project '{proj}' --parameter audit-param")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(2, entries.len());
    assert_eq!(2, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Parameter").count());
    assert_eq!(1, entries.find_by_type("Value").count());

    let cmd = cloudtruth!("audit-logs ls -f json -m 20 --env {env}")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(2, entries.len());
    assert_eq!(2, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Value").count());
    assert_eq!(1, entries.find_by_type("Environment").count());

    /* Bad filters */
    cloudtruth!("audit-logs ls --env my-bogus-env")
        .envs(&env_map)
        .assert()
        .failure()
        .stderr(contains!("Environment 'my-bogus-env' not found"));
    cloudtruth!("audit-logs ls --project my-bogus-proj")
        .envs(&env_map)
        .assert()
        .failure()
        .stderr(contains!("Project 'my-bogus-proj' not found"));
    cloudtruth!("audit-logs ls --project '{proj}' --parameter my-bogus-param")
        .envs(&env_map)
        .assert()
        .failure()
        .stderr(contains!("Parameter 'my-bogus-param' not found"));
    cloudtruth!("audit-logs ls --parameter audit-param")
        .envs(&env_map)
        .assert()
        .failure()
        .stderr(contains!(
            "Must specify a project when specifying a parameter"
        ));

    /* delete stuff */
    cloudtruth!("template delete my-audit-template --confirm")
        .envs(&env_map)
        .assert()
        .success();
    cloudtruth!("param delete audit-param --confirm")
        .envs(&env_map)
        .assert()
        .success();
    drop(proj);
    let old_env_name = env.name().as_str().to_string(); // save old env name for later
    drop(env);

    /* recreate environment vars */
    let env_map = hashmap! {
        CT_API_KEY => test_user.api_key()
    };

    let cmd = cloudtruth!("audit ls -f json -t parameter")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(entries.len(), entries.find_by_type("Parameter").count());
    let (create_count, delete_count) = entries.get_create_delete_count("Parameter", "audit-param");
    assert_ne!(0, create_count);
    assert_ne!(0, delete_count);

    let cmd = cloudtruth!("audit ls -f json -t template")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(entries.len(), entries.find_by_type("Template").count());
    let (create_count, delete_count) = cmd
        .get_audit_log_entries()
        .get_create_delete_count("Template", "my-audit-template");
    assert_ne!(0, create_count);
    assert_ne!(0, delete_count);

    let cmd = cloudtruth!("audit ls -f json -t environment")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(entries.len(), entries.find_by_type("Environment").count());
    let (create_count, delete_count) =
        entries.get_create_delete_count("Environment", &old_env_name);
    assert_ne!(0, create_count);
    assert_ne!(0, delete_count);

    let cmd = cloudtruth!("audit ls -f json -t value")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.get_audit_log_entries();
    assert_eq!(entries.len(), entries.find_by_type("Value").count());
    let (create_count, delete_count) = entries.get_create_delete_count("Value", &old_env_name);
    assert_eq!(0, create_count);
    assert_eq!(0, delete_count);

    for object_type in [
        "AwsIntegration",
        "Environment",
        "GitHubIntegration",
        "Invitation",
        "Membership",
        "Organization",
        "Parameter",
        "ParameterRule",
        "ParameterType",
        "ParameterTypeRule",
        "Project",
        "Pull",
        "Push",
        "ServiceAccount",
        "Tag",
        "Task",
        "Template",
        "Value",
    ] {
        let cmd = cloudtruth!("audit ls -f json -m 5 -t {object_type}")
            .envs(&env_map)
            .assert()
            .success();
        let entries = cmd.get_audit_log_entries();
        assert!(5 >= entries.len());
        assert_eq!(entries.len(), entries.find_by_type(object_type).count());
    }

    /* time filtered */
    // nothing found for old date
    cloudtruth!("audit ls -m 1 --before 2021-10-31")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(starts_with("No audit log entries"));
    // nothing found for date in the future
    cloudtruth!("audit ls -m 1 --after 8084-10-31")
        .envs(&env_map)
        .assert()
        .success()
        .stdout(starts_with("No audit log entries"));
}

use cloudtruth_config::{CT_API_KEY, CT_ENVIRONMENT, CT_PROJECT};
use cloudtruth_test_harness::prelude::*;
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
    let summary = cmd.get_output();
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
    let entries = cmd.parse_audit_log_json();
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
    let entries = cmd.parse_audit_log_json();
    assert_eq!(2, entries.len());
    assert_eq!(2, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Parameter").count());
    assert_eq!(1, entries.find_by_type("Value").count());

    let cmd = cloudtruth!("audit-logs ls -f json -m 20 --env {env}")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.parse_audit_log_json();
    assert_eq!(2, entries.len());
    assert_eq!(2, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Value").count());
    assert_eq!(1, entries.find_by_type("Environment").count());

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
    let entries = cmd.parse_audit_log_json();
    assert_eq!(entries.len(), entries.find_by_type("Parameter").count());
    let (create_count, delete_count) = entries.get_create_delete_count("Parameter", "audit-param");
    assert_ne!(0, create_count);
    assert_ne!(0, delete_count);

    let cmd = cloudtruth!("audit ls -f json -t template")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.parse_audit_log_json();
    assert_eq!(entries.len(), entries.find_by_type("Template").count());
    let (create_count, delete_count) = cmd
        .parse_audit_log_json()
        .get_create_delete_count("Template", "my-audit-template");
    assert_ne!(0, create_count);
    assert_ne!(0, delete_count);

    let cmd = cloudtruth!("audit ls -f json -t environment")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.parse_audit_log_json();
    assert_eq!(entries.len(), entries.find_by_type("Environment").count());
    let (create_count, delete_count) =
        entries.get_create_delete_count("Environment", &old_env_name);
    assert_ne!(0, create_count);
    assert_ne!(0, delete_count);

    let cmd = cloudtruth!("audit ls -f json -t value")
        .envs(&env_map)
        .assert()
        .success();
    let entries = cmd.parse_audit_log_json();
    assert_eq!(entries.len(), entries.find_by_type("Value").count());
    let (create_count, delete_count) = entries.get_create_delete_count("Value", &old_env_name);
    assert_eq!(0, create_count);
    assert_eq!(0, delete_count);

    // parse audit summary snapshot
    let summary = AuditLogSummary::parse(&summary.stdout);
    let entries = cloudtruth!(
        "audit ls -f json -m 1 --before {timestamp}",
        timestamp = summary.earliest_record().to_rfc3339()
    )
    .envs(&env_map)
    .assert()
    .success()
    .parse_audit_log_json();
    assert!(entries.len() < summary.record_count());
    let newer = entries
        .iter()
        .filter(|e| &e.time > summary.earliest_record());
    assert_eq!(0, newer.count());
    let entries = cloudtruth!("audit ls -f json -m 3")
        .envs(&env_map)
        .assert()
        .success()
        .parse_audit_log_json();
    let after = entries.last().unwrap().time;
    let entries = cloudtruth!(
        "audit ls -f json -m 1 --after {ts}",
        ts = after.to_rfc3339()
    )
    .envs(&env_map)
    .assert()
    .success()
    .parse_audit_log_json();
    assert!(entries.len() < summary.record_count());
    let older = entries.iter().filter(|e| e.time < after);
    assert_eq!(older.count(), 0);

    // compare snapshot summary with latest summary
    let cmd = cloudtruth!("audit summary").assert().success();
    let final_summary = AuditLogSummary::parse(&cmd.get_output().stdout);
    assert_ne!(summary, final_summary);
}

#[use_harness]
#[test]
fn test_audit_logs_basic() {
    let entries = cloudtruth!("audit ls -f json")
        .assert()
        .success()
        .parse_audit_log_json();
    assert_ne!(0, entries.len());
}
#[use_harness]
#[test]
fn test_audit_logs_time_filters() {
    /* time filtered */
    // nothing found for old date
    cloudtruth!("audit ls -m 1 --before 2021-10-31")
        .assert()
        .success()
        .stdout(starts_with("No audit log entries"));
    // nothing found for date in the future
    cloudtruth!("audit ls -m 1 --after 8084-10-31")
        .assert()
        .success()
        .stdout(starts_with("No audit log entries"));
}

#[use_harness]
#[test]
fn test_audit_logs_type_filters() {
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
            .assert()
            .success();
        let entries = cmd.parse_audit_log_json();
        assert!(5 >= entries.len());
        assert_eq!(entries.len(), entries.find_by_type(object_type).count());
    }
}

#[use_harness]
#[test]
fn test_audit_logs_bad_filters() {
    cloudtruth!("audit ls --before foo")
        .assert()
        .failure()
        .stderr(contains!("Invalid '--before' value"));
    cloudtruth!("audit ls --after bar")
        .assert()
        .failure()
        .stderr(contains!("Invalid '--after' value"));
    cloudtruth!("audit ls --after bar --before foo")
        .assert()
        .failure()
        .stderr(contains_all([
            "Invalid '--before' value",
            "Invalid '--after' value",
        ]));
    cloudtruth!("audit ls --user 'ricardo.multiban'")
        .assert()
        .failure()
        .stderr(contains!("User 'ricardo.multiban' not found"));
    cloudtruth!("audit ls --type snafoo")
        .assert()
        .success()
        .stderr(contains!(
            "The specified --type is not one of the recognized values"
        ));
    cloudtruth!("audit-logs ls --env my-bogus-env")
        .assert()
        .failure()
        .stderr(contains!("Environment 'my-bogus-env' not found"));
    cloudtruth!("audit-logs ls --project my-bogus-proj")
        .assert()
        .failure()
        .stderr(contains!("Project 'my-bogus-proj' not found"));
    Project::with_prefix("audit-log-bad-filters").with_scope(|proj| {
        cloudtruth!("audit-logs ls --project '{proj}' --parameter my-bogus-param")
            .assert()
            .failure()
            .stderr(contains!("Parameter 'my-bogus-param' not found"));
    });
    cloudtruth!("audit-logs ls --parameter audit-param")
        .assert()
        .failure()
        .stderr(contains!(
            "Must specify a project when specifying a parameter"
        ));
}

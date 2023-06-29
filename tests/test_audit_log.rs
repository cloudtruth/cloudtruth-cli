use std::collections::HashMap;

use cloudtruth_config::{CT_API_KEY, CT_ENVIRONMENT, CT_PROJECT};
use integration_test_harness::prelude::*;

#[use_harness]
#[test]
fn test_audit_logs() {
    /* Environment variables map */
    let mut env_map = HashMap::new();
    /* Create admin user for testing */
    let test_user = User::with_prefix("log-user").role("admin").create();
    env_map.insert(CT_API_KEY, test_user.api_key());
    /* Take a summary snapshot */
    // let summary = cloudtruth!("audit summary")
    //     .envs(&env_map)
    //     .assert()
    //     .success();
    // let summary = summary.get_output();
    /* Create test project */
    let proj = Project::with_prefix("audit-proj").create();
    env_map.insert(CT_PROJECT, proj.name().as_str());
    /* Create test environment */
    let env = Environment::with_prefix("audit-env").create();
    env_map.insert(CT_ENVIRONMENT, env.name().as_str());

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

    let entries = cloudtruth!("audit-logs ls -f json -m 20 --project '{proj}'")
        .envs(&env_map)
        .assert()
        .success();
    let entries: AuditLogEntries = entries.get_audit_log_entries();
    assert_eq!(4, entries.len());
    assert_eq!(4, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Project").count());
    assert_eq!(1, entries.find_by_type("Parameter").count());
    assert_eq!(1, entries.find_by_type("Template").count());
    assert_eq!(1, entries.find_by_type("Value").count());
    let entries =
        cloudtruth!("audit-logs ls -f json -m 20 --project '{proj}' --parameter audit-param")
            .envs(&env_map)
            .assert()
            .success();
    let entries: AuditLogEntries = entries.get_audit_log_entries();
    assert_eq!(2, entries.len());
    assert_eq!(2, entries.find_by_action("create").count());
    assert_eq!(1, entries.find_by_type("Parameter").count());
    assert_eq!(1, entries.find_by_type("Value").count());
    let entries = cloudtruth!("audit-logs ls -f json -m 20 --env {env}")
        .envs(&env_map)
        .assert()
        .success();
    let entries: AuditLogEntries = entries.get_audit_log_entries();
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
}

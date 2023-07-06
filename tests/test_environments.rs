use cloudtruth_test_harness::prelude::*;
use std::str;

const TEST_PAGE_SIZE: usize = 5;

#[test]
#[use_harness]
fn test_environment_basic() {
    // Initialize environment data but do not create yet
    let env = Environment::with_prefix("env-name").description("Description on create");

    // verify env does not yet exist
    cloudtruth!("environments ls -v")
        .assert()
        .success()
        .stdout(not(contains(env.name())));

    let mut env = env.create();

    // create/delete within scope of this closure
    cloudtruth!("environments ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{env},default,Description on create"));

    // update the description
    cloudtruth!("environments set {env} --desc 'Updated description'")
        .assert()
        .success();

    cloudtruth!("environments ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{env},default,Updated description"));

    // idempotent - do it again
    cloudtruth!("environments set {env} --desc 'Updated description'")
        .assert()
        .success();

    // rename
    env.rename(Name::with_prefix("env-rename"));

    // nothing to update
    cloudtruth!("environments set {env}")
        .assert()
        .success()
        .stderr(contains!(
            "Environment '{env}' not updated: no updated parameters provided"
        ));

    // test the list without the values
    cloudtruth!("environments list")
        .assert()
        .success()
        .stdout(contains(env.name()).and(not(contains("Updated description"))));

    // shows create/modified times
    cloudtruth!("environments list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(env.name()))
                .and(contains("Updated description")),
        );

    //explicitly delete
    let deleted_env = Environment::from_name(env.name().clone());
    drop(env);
    // verify deletion
    cloudtruth!("environments ls -v")
        .assert()
        .success()
        .stdout(not(contains(deleted_env.name())));

    // do it again, see we have success and a warning
    cloudtruth!("environments delete {deleted_env} --confirm")
        .assert()
        .success()
        .stderr(contains!("Environment '{deleted_env}' does not exist"));
}

#[test]
#[use_harness]
fn test_environment_cannot_delete_default() {
    let proj = Project::with_prefix("default-env-del").create();

    // get snapshot of params before attempting to delete
    let before_param_list = cloudtruth!("--project {proj} --env default param ls -v")
        .assert()
        .success();

    cloudtruth!("environments delete default --confirm")
        .assert()
        .failure()
        .stderr(contains("Cannot delete the default environment"));

    cloudtruth!("--project {proj} --env default param ls -v")
        .assert()
        .success()
        .stdout(diff(
            String::from_utf8_lossy(&before_param_list.get_output().stdout).to_string(),
        ))
        .stderr(diff(
            String::from_utf8_lossy(&before_param_list.get_output().stderr).to_string(),
        ))
        .code(eq(before_param_list
            .get_output()
            .status
            .code()
            .expect("No status code")));
}

#[test]
#[use_harness]
fn test_environment_copy() {
    let env = Environment::with_prefix("env-copy-src").create();
    let env2 = env.copy(Name::with_prefix("env-copy-dest"));
    cloudtruth!("env ls")
        .assert()
        .success()
        .stdout(contains(env.name()).and(contains(env2.name())));
}

#[test]
#[use_harness]
fn test_environment_parents() {
    let env1 = Environment::with_prefix("env-par-1").create();
    let env2 = Environment::with_prefix("env-mid-1").parent(&env1).create();
    let env3 = Environment::with_prefix("env-chld-3")
        .parent(&env2)
        .create();
    let env4 = Environment::with_prefix("env-chld-4")
        .parent(&env2)
        .create();

    // Use csv to validate, since the names may be variable
    cloudtruth!("env ls -v -f csv").assert().success().stdout(
        contains!("{env1},default,")
            .and(contains!("{env2},{env1},"))
            .and(contains!("{env3},{env2},"))
            .and(contains!("{env4},{env2},")),
    );

    cloudtruth!("env tree").assert().success().stdout(contains!(
        "  {env1}\n    {env2}\n      {env3}\n      {env4}\n"
    ));

    // specifying the environment gets a filtered set
    cloudtruth!("env tree {env2}")
        .assert()
        .success()
        .stdout(contains!("{env2}\n  {env3}\n  {env4}\n"));

    // Invalid environment given
    cloudtruth!("env tree 'invalid-env'")
        .assert()
        .success()
        .stderr(diff("No environment 'invalid-env' found\n"));

    // Attempt to delete an environment that is used somewhere
    cloudtruth!("env delete {env2} --confirm")
        .assert()
        .failure()
        .stderr(
            contains("Cannot remove environment because it has children")
                .and(contains(env3.name()).and(contains(env4.name()))),
        );

    let env5 = Environment::with_prefix("env-par-5");
    let env6 = Environment::with_prefix("env-par-6");

    // attempt to create without an existing parent
    cloudtruth!("env set {env5} --parent {env6}")
        .assert()
        .failure()
        .stderr(contains!("No parent environment '{env6}' found\n"));

    // attempt to update parent -- not allowed
    cloudtruth!("environment set {env4} --parent {env1}")
        .assert()
        .failure()
        .stderr(diff!("Environment '{env4}' parent cannot be updated.\n"));

    // setting to same parent is ignored
    cloudtruth!("environment set {env4} --parent {env2} --desc 'My new description'")
        .assert()
        .success();

    cloudtruth!("environment ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{env4},{env2},My new description"));
}

#[test]
#[use_harness]
fn test_environment_pagination() {
    let page_size = TEST_PAGE_SIZE;
    // we store the project names so they're not instantly dropped and deleted
    let _envs: Vec<Scope<Environment>> = (0..=page_size)
        .map(|i| Environment::with_prefix(format!("env-page-{}", i)).create())
        .collect();
    cloudtruth!("env ls")
        .rest_debug()
        .page_size(page_size)
        .assert()
        .success()
        .paginated(page_size);
}

#[test]
#[use_harness]
fn test_environment_tagging() {
    let proj = Project::with_prefix("proj-env-tag").create();
    let env = Environment::with_prefix("env-tag").create();

    cloudtruth!("--env {env} --project {proj} param set my-param -v 'temp value'")
        .assert()
        .success();

    // make sure env list is empty
    for format_opts in ["", "-v", "-f csv", "-vf csv"] {
        cloudtruth!("env tag list {env} {format_opts}")
            .assert()
            .success()
            .stdout(contains!("No tags found in environment {env}"));
    }

    cloudtruth!("env tag set {env} my-tag -d 'first description'")
        .assert()
        .success();

    // make sure we can see the tag, and save the timestamp
    let cmd = cloudtruth!("env tag list {env} -vf csv")
        .assert()
        .success()
        .stdout(contains("my-tag").and(contains("first description")));
    let timestamp = str::from_utf8(&cmd.get_output().stdout)
        .unwrap()
        .lines()
        .nth(1)
        .unwrap()
        .split(',')
        .nth(1)
        .unwrap();

    // update the description
    cloudtruth!("env tag set {env} my-tag -d 'updated description'")
        .assert()
        .success();

    // see description was updated and timestamp is preserved
    cloudtruth!("env tag list {env} -v")
        .assert()
        .success()
        .stdout(contains("updated description").and(contains(timestamp)));

    // rename the tag
    cloudtruth!("env tag set {env} my-tag --rename renamed-tag")
        .assert()
        .success();

    // set a timestamp
    cloudtruth!("env tag set {env} renamed-tag -t 03/24/2021")
        .assert()
        .success();

    // warning when nothing is updated
    cloudtruth!("env tag set {env} renamed-tag")
        .assert()
        .success()
        .stderr(diff(
            "Nothing changed. Please provide description, time, immutable, mutable, or current.\n",
        ));

    // cannot use --current and --time at the same time
    cloudtruth!("env tag set {env} renamed-tag --current --time 2021-10-01")
        .assert()
        .failure()
        .stderr(contains("Conflicting arguments: cannot specify both"));

    // Invalid timestamps
    for timestamp in ["abcd", "2000"] {
        cloudtruth!("env tag set {env} renamed-tag -t '{timestamp}'")
            .assert()
            .failure()
            .stderr(diff(
                "Invalid time value -- use an accepted timestamp format\n",
            ));
    }

    // delete the tag
    cloudtruth!("env tag del {env} renamed-tag --confirm")
        .assert()
        .success();

    // idempotent
    cloudtruth!("env tag del {env} renamed-tag --confirm")
        .assert()
        .success()
        .stderr(diff!(
            "Environment '{env}' does not have a tag 'renamed-tag'!\n"
        ));

    // unknown environment
    cloudtruth!("env tag list invalid-env")
        .assert()
        .failure()
        .stderr(diff(
            "The 'invalid-env' environment could not be found in your account.\n",
        ));
    cloudtruth!("env tag set invalid-env my-tag")
        .assert()
        .failure()
        .stderr(diff(
            "The 'invalid-env' environment could not be found in your account.\n",
        ));
    cloudtruth!("env tag delete invalid-env my-tag")
        .assert()
        .success()
        .stderr(diff("Environment 'invalid-env' does not exist!\n"));
}

#[test]
#[use_harness]
fn test_environment_tagging_pagination() {
    let env = Environment::with_prefix("env-pag-tag").create();

    let page_size = TEST_PAGE_SIZE;
    for n in 0..=page_size {
        let tag = Name::with_prefix(format!("tag-{n}"));
        cloudtruth!("env tag set {env} {tag}").assert().success();
    }
    cloudtruth!("env tag ls {env}")
        .rest_debug()
        .page_size(page_size)
        .assert()
        .success()
        .paginated(page_size);
}

#[test]
#[use_harness]
fn test_environment_tagging_immutable() {
    let env = Environment::with_prefix("env-tag-immutable").create();

    cloudtruth!("env tag set --immutable --current {env} immutable-tag")
        .assert()
        .success()
        .stdout(contains!(
            "Created tag 'immutable-tag' in environment '{env}'"
        ));

    cloudtruth!("env tag set --time 05/24/2023 {env} immutable-tag")
        .assert()
        .failure()
        .stderr(contains("Tag immutable-tag is immutable"));
}

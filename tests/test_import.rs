use integration_test_harness::prelude::*;

const TEXT1: &str = r#"
MY_PARAM='this "contains" quotes'
MY_SECRET=password
PARAM1='updated value'
PARAM2='UNREFERENCED = going away'
secret_2='sssshhhhh'
STRING_PARAM=

"#;

const TEXT2: &str = r#"
MY_PARAM="no quotes here"
MY_SECRET="password"
PARAM1="my workspace"
PARAM2="UNREFERENCED = going away"
STRING_PARAM=""
secret_2="be veewwy quiet"

"#;

#[test]
#[use_harness]
fn test_import_basic() {
    let f = TestFile::with_contents(TEXT1).unwrap();
    // Scope::new is used to add to cleanup without creating
    let proj = Scope::new(Project::with_prefix("proj-import"));
    let env1 = Scope::new(Environment::with_prefix("env-import"));
    let env2 = Environment::with_prefix("env-import-child").parent(&env1);

    //verify project and environment does not exist
    cloudtruth!("proj ls")
        .assert()
        .success()
        .stdout(not(contains(proj.name())));
    cloudtruth!("env ls")
        .assert()
        .success()
        .stdout(not(contains(env1.name())));

    // preview
    cloudtruth!(
        "import param {proj} {f} --env {env1} --secret MY_SECRET --secret secret_2 --preview"
    )
    .assert()
    .success();

    cloudtruth!("import param {proj} {f} --env {env1} --secret MY_SECRET --secret secret_2 --preview --format json")
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("MY_PARAM")),
                prop("Value", value(r#"this "contains" quotes"#)),
            ),
            find_entry(
                prop("Name", value("MY_SECRET")),
                prop("Value", value("*****")),
            ),
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("updated value")),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away")),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("*****")),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")),
            ),
        )
        .and(for_all(all!(
            prop("Project", value(proj.to_name())),
            prop("Environment", value(env1.to_name())),
            prop("Change", value("created")),
        ))),
    )));
    // preview again with secrets
    cloudtruth!("import param {proj} {f} --env {env1} --secret MY_SECRET --secret secret_2 --preview --format json --secrets")
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("MY_PARAM")),
                prop("Value", value(r#"this "contains" quotes"#)),
            ),
            find_entry(
                prop("Name", value("MY_SECRET")),
                prop("Value", value("password")),
            ),
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("updated value")),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away")),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("sssshhhhh")),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")),
            ),
        )
        .and(for_all(all!(
            prop("Project", value(proj.to_name())),
            prop("Environment", value(env1.to_name())),
            prop("Change", value("created")),
        ))),
    )));

    //verify project and environment does not exist
    cloudtruth!("proj ls")
        .assert()
        .success()
        .stdout(not(contains(proj.name())));
    cloudtruth!("env ls")
        .assert()
        .success()
        .stdout(not(contains(env1.name())));

    // do the first import
    cloudtruth!("import param {proj} {f} --env {env1} --secret MY_SECRET --secret secret_2 --format json --secrets")
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("MY_PARAM")),
                prop("Value", value(r#"this "contains" quotes"#)),
            ),
            find_entry(
                prop("Name", value("MY_SECRET")),
                prop("Value", value("password")),
            ),
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("updated value")),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away")),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("sssshhhhh")),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")),
            ),
        )
        .and(for_all(all!(
            prop("Project", value(proj.to_name())),
            prop("Environment", value(env1.to_name())),
            prop("Change", value("created")),
        ))),
    )));

    //verify project and environment were created
    cloudtruth!("proj ls")
        .assert()
        .success()
        .stdout(contains(proj.name()));
    cloudtruth!("env ls")
        .assert()
        .success()
        .stdout(contains(env1.name()));

    // verify the parameters
    cloudtruth!("--project {proj} --env {env1} param ls -s -f json")
        .assert()
        .success()
        .stdout(json(prop(
            "parameter",
            all!(
                find_entry(
                    prop("Name", value("MY_PARAM")),
                    prop("Value", value(r#"this "contains" quotes"#))
                        .and(prop("Secret", value("false"))),
                ),
                find_entry(
                    prop("Name", value("MY_SECRET")),
                    prop("Value", value("password")).and(prop("Secret", value("true"))),
                ),
                find_entry(
                    prop("Name", value("PARAM1")),
                    prop("Value", value("updated value")).and(prop("Secret", value("false"))),
                ),
                find_entry(
                    prop("Name", value("PARAM2")),
                    prop("Value", value("UNREFERENCED = going away"))
                        .and(prop("Secret", value("false"))),
                ),
                find_entry(
                    prop("Name", value("secret_2")),
                    prop("Value", value("sssshhhhh")).and(prop("Secret", value("true"))),
                ),
                find_entry(
                    prop("Name", value("STRING_PARAM")),
                    prop("Value", value("")).and(prop("Secret", value("false"))),
                ),
            ),
        )));

    // redo -- no changes
    cloudtruth!(
        "import param {proj} {f} --env {env1} --secret MY_SECRET --secret secret_2 --format json"
    )
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("MY_PARAM")),
                prop("Value", value(r#"this "contains" quotes"#)),
            ),
            find_entry(
                prop("Name", value("MY_SECRET")),
                prop("Value", value("*****")),
            ),
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("updated value")),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away")),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("*****")),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")),
            ),
        )
        .and(for_all(all!(
            prop("Project", value(proj.to_name())),
            prop("Environment", value(env1.to_name())),
            prop("Change", value("unchanged")),
        ))),
    )));

    // use a different text file
    let f = TestFile::with_contents(TEXT2).unwrap();

    // preview with secrets
    cloudtruth!("import param {proj} {f} --env {env1} --secret MY_SECRET --secret secret_2 --preview --format json --secrets")
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("MY_PARAM")),
                prop("Value", value("no quotes here")).and(prop("Change", value("updated"))),
            ),
            find_entry(
                prop("Name", value("MY_SECRET")),
                prop("Value", value("password")).and(prop("Change", value("unchanged"))),
            ),
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("my workspace")).and(prop("Change", value("updated"))),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away")).and(prop("Change", value("unchanged"))),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("be veewwy quiet")).and(prop("Change", value("updated"))),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")).and(prop("Change", value("unchanged"))),
            ),
        )
    )));

    let env2 = env2.create();
    cloudtruth!("import param {proj} {f} --env {env2} --preview --format json --secrets")
        .assert()
        .success()
        .stdout(json(prop(
            "parameter",
            all!(
                find_entry(
                    prop("Name", value("MY_PARAM")),
                    prop("Value", value("no quotes here")).and(prop("Change", value("overridden"))),
                ),
                find_entry(
                    prop("Name", value("MY_SECRET")),
                    prop("Value", value("password")).and(prop("Change", value("inherited"))),
                ),
                find_entry(
                    prop("Name", value("PARAM1")),
                    prop("Value", value("my workspace")).and(prop("Change", value("overridden"))),
                ),
                find_entry(
                    prop("Name", value("PARAM2")),
                    prop("Value", value("UNREFERENCED = going away"))
                        .and(prop("Change", value("inherited"))),
                ),
                find_entry(
                    prop("Name", value("secret_2")),
                    prop("Value", value("be veewwy quiet"))
                        .and(prop("Change", value("overridden"))),
                ),
                find_entry(
                    prop("Name", value("STRING_PARAM")),
                    prop("Value", value("")).and(prop("Change", value("inherited"))),
                ),
            ),
        )));
    // no-inherit
    cloudtruth!(
        "import param {proj} {f} --env {env2} --preview --format json --secrets --no-inherit"
    )
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("MY_PARAM")),
                prop("Value", value("no quotes here")).and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("MY_SECRET")),
                prop("Value", value("password")).and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("my workspace")).and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away"))
                    .and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("be veewwy quiet")).and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")).and(prop("Change", value("overridden"))),
            ),
        ),
    )));
    // --ignore
    cloudtruth!(
        "import param {proj} {f} --env {env2} --ignore MY_PARAM --ignore MY_SECRET --preview --format json --secrets"
    )
    .assert()
    .success()
    .stdout(json(prop(
        "parameter",
        all!(
            find_entry(
                prop("Name", value("PARAM1")),
                prop("Value", value("my workspace")).and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("PARAM2")),
                prop("Value", value("UNREFERENCED = going away"))
                    .and(prop("Change", value("inherited"))),
            ),
            find_entry(
                prop("Name", value("secret_2")),
                prop("Value", value("be veewwy quiet")).and(prop("Change", value("overridden"))),
            ),
            find_entry(
                prop("Name", value("STRING_PARAM")),
                prop("Value", value("")).and(prop("Change", value("inherited"))),
            ),
        ),
    )));
}

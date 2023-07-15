use cloudtruth_config::{CT_ENVIRONMENT, CT_PROJECT};
use cloudtruth_test_harness::output::profile::get_current_user;
use cloudtruth_test_harness::output::template::*;
use cloudtruth_test_harness::prelude::*;
use maplit::hashmap;

#[test]
#[use_harness]
fn test_templates_basic() {
    let proj = Project::with_prefix("template-proj").create();

    let temp_file = TestFile::with_contents("Text with no params\n").unwrap();

    let temp_name = Name::with_prefix("orig-template");

    cloudtruth!("--project {proj} template ls -v")
        .assert()
        .success()
        .stdout(contains!("No templates in project '{proj}'"));

    // Create template
    cloudtruth!(
        "--project {proj} templates set {temp_name} --desc 'Description on create' --body '{temp_file}' "
    )
    .assert()
    .success()
    .stdout(contains!("Created template '{temp_name}'"));

    cloudtruth!("--project {proj} template ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{temp_name},Description on create"));

    cloudtruth!("--project {proj} template get {temp_name}")
        .assert()
        .success()
        .stdout("Text with no params\n");

    cloudtruth!("--project {proj} template validate {temp_name}")
        .assert()
        .success()
        .stdout(contains("Success"));

    // Update the description
    cloudtruth!("--project {proj} template set {temp_name} --desc 'Updated description'")
        .assert()
        .success()
        .stdout(contains!("Updated template '{temp_name}'"));

    cloudtruth!("--project {proj} template ls --values -f csv")
        .assert()
        .success()
        .stdout(contains!("{temp_name},Updated description"));

    // idempotent - do it again
    cloudtruth!("--project {proj} template set {temp_name} --desc 'Updated description'")
        .assert()
        .success();

    // rename
    let orig_name = temp_name;
    let temp_name = Name::with_prefix("renamed-temp");
    cloudtruth!("--project {proj} template set {orig_name} --rename {temp_name}")
        .assert()
        .success()
        .stdout(contains!("Updated template '{temp_name}'"));

    cloudtruth!("--project {proj} template ls")
        .assert()
        .success()
        .stdout(contains(&temp_name).and(not(contains(&orig_name))));

    // attempting to get template that does not exist yield error
    cloudtruth!("--project {proj} template get {orig_name}")
        .assert()
        .failure()
        .stderr(contains!(
            "No template '{orig_name}' found in project '{proj}'"
        ));

    // change the body
    let body = TestFile::with_contents("different fixed value\n").unwrap();

    cloudtruth!("--project {proj} template set {temp_name} --body '{body}'")
        .assert()
        .success()
        .stdout(contains!("Updated template '{temp_name}'"));

    cloudtruth!("--project {proj} template get {temp_name}")
        .assert()
        .success()
        .stdout(diff("different fixed value\n"));

    cloudtruth!("--project {proj} template get {temp_name} --raw")
        .assert()
        .success()
        .stdout(diff("different fixed value\n"));

    // nothing to update
    cloudtruth!("--project {proj} template set {temp_name}")
        .assert()
        .success()
        .stderr(contains!(
            "Template '{temp_name}' not updated: no updated parameters provided"
        ));

    cloudtruth!("--project {proj} templates list")
        .assert()
        .success()
        .stdout(contains(&temp_name).and(not(contains("Updated description"))));

    // delete template
    cloudtruth!("--project {proj} template delete {temp_name} --confirm")
        .assert()
        .success();

    // try to delete again
    cloudtruth!("--project {proj} template delete {temp_name} --confirm")
        .assert()
        .success()
        .stderr(contains!(
            "Template '{temp_name}' does not exist for project '{proj}'"
        ));
}

#[test]
#[use_harness]
fn test_templates_evaluate_environments() {
    let proj = Project::with_prefix("temp-eval").create();
    let env1 = Environment::with_prefix("env_eval_a").create();
    let env2 = Environment::with_prefix("env-eval_b").create();

    let mut vars = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };

    cloudtruth!("param set param1 --value 'some val with space'")
        .envs(&vars)
        .env(CT_ENVIRONMENT, env1.name().as_str())
        .assert()
        .success();
    cloudtruth!("param set param1 --value diff_env_value")
        .envs(&vars)
        .env(CT_ENVIRONMENT, env2.name().as_str())
        .assert()
        .success();
    cloudtruth!("param set secret1 --secret true --value sssshhhhhhh")
        .envs(&vars)
        .env(CT_ENVIRONMENT, env1.name().as_str())
        .assert()
        .success();
    cloudtruth!("param set secret1 --secret true --value top-secret")
        .envs(&vars)
        .env(CT_ENVIRONMENT, env2.name().as_str())
        .assert()
        .success();

    let template_text = "\
        # here is a comment\n\
        // we do not care about what other content you put in\n\
        simple.param={{param1}}\n\
        ANOTHER_PARAM={{secret1}}\n\
    ";
    let test_file = TestFile::with_contents(template_text).unwrap();

    let temp_name = Name::with_prefix("eval-env-temp");
    cloudtruth!("template set {temp_name} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();

    /* check environment 1 */
    vars.insert(CT_ENVIRONMENT, env1.name().as_str());

    // evaluated template hides secrets
    cloudtruth!("template get {temp_name}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains("*****"));

    // check evaluation
    cloudtruth!("template get {temp_name} -s")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # here is a comment\n\
            // we do not care about what other content you put in\n\
            simple.param=some val with space\n\
            ANOTHER_PARAM=sssshhhhhhh\n\
            ",
        ));

    // check raw
    cloudtruth!("template get -r {temp_name}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(template_text));

    //check preview without secrets
    cloudtruth!("template preview {test_file}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains("****"));

    //check preview with secrets
    cloudtruth!("template preview {test_file} -s")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # here is a comment\n\
            // we do not care about what other content you put in\n\
            simple.param=some val with space\n\
            ANOTHER_PARAM=sssshhhhhhh\n\
            ",
        ));

    /* check environment 2 */
    vars.insert(CT_ENVIRONMENT, env2.name().as_str());

    // evaluated template hides secrets
    cloudtruth!("template get {temp_name}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains("*****"));

    // check evaluation
    cloudtruth!("template get {temp_name} -s")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # here is a comment\n\
            // we do not care about what other content you put in\n\
            simple.param=diff_env_value\n\
            ANOTHER_PARAM=top-secret\n\
            ",
        ));

    // check raw
    cloudtruth!("template get -r {temp_name}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(template_text));

    //check preview without secrets
    cloudtruth!("template preview {test_file}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains("****"));

    //check preview with secrets
    cloudtruth!("template preview {test_file} -s")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # here is a comment\n\
            // we do not care about what other content you put in\n\
            simple.param=diff_env_value\n\
            ANOTHER_PARAM=top-secret\n\
            ",
        ));

    // see that we cannot delete a parameter with the template using it
    cloudtruth!("param del -y param1")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(contains!(
            "Cannot delete param1 because it is referenced by the following templates: {temp_name}"
        ));

    // check error messages with unresolved variables
    let template_text = template_text.replace("{{param1}}", "{{no_param}}");
    let test_file = TestFile::with_contents(template_text).unwrap();
    cloudtruth!("template preview {test_file} --secrets")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(
            contains("Template contains references that do not exist").and(contains("no_param")),
        );
}

#[test]
#[use_harness]
fn test_templates_as_of_time() {
    let proj = Project::with_prefix("temp-times").create();

    let vars = hashmap! {
        CT_PROJECT => proj.name().as_str()
    };

    cloudtruth!("param set some_param --value 'value first'")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("param set another_param --value 'devops'")
        .envs(&vars)
        .assert()
        .success();

    let test_temp = Name::with_prefix("temp-times");
    let template_text1 = "\
    # just a different template\n\
    references = {{some_param}}\n\
    ";
    let test_file = TestFile::with_contents(template_text1).unwrap();

    cloudtruth!("template set {test_temp} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();

    let cmd = cloudtruth!("template list --show-times -f json")
        .envs(&vars)
        .assert()
        .success();
    let modified_at = cmd.get_template_modified_at(0);
    cloudtruth!("param set some_param --value 'value second'")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("param set another_param --value 'sre'")
        .envs(&vars)
        .assert()
        .success();

    let template_text2 = "\
    # just a different template\n\
    references = {{another_param}}\n\
    ";
    let test_file = TestFile::with_contents(template_text2).unwrap();
    cloudtruth!("template set {test_temp} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("template get {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a different template\n\
            references = sre\n\
            ",
        ));

    cloudtruth!("template get --raw {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(template_text2));

    cloudtruth!("template get --as-of '{modified_at}' {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a different template\n\
            references = value first\n\
            ",
        ));

    cloudtruth!("template get --raw --as-of '{modified_at}' {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(template_text1));

    //before project exists
    cloudtruth!("template get --as-of '2020-02-02' {test_temp}")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(contains(
            "Did not find environment 'default' at specified time/tag",
        ));

    // check preview
    let preview_template_text = "\
        # just a comment\n\
        this.is.a.template.value={{some_param}}\n\
    ";
    let preview_file = TestFile::with_contents(preview_template_text).unwrap();
    cloudtruth!("template set {test_temp} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();
    cloudtruth!("template preview {preview_file}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a comment\n\
            this.is.a.template.value=value second\n\
            ",
        ));
    cloudtruth!("template preview {preview_file} --as-of '{modified_at}'")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a comment\n\
            this.is.a.template.value=value first\n\
            ",
        ));
    //before project exists
    cloudtruth!("template preview {preview_file} --as-of 2020-02-02")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(contains("No ProjectLedger matches the given query"));
}

#[test]
#[use_harness]
fn test_templates_as_of_tag() {
    let proj = Project::with_prefix("temp-tag").create();
    let env = Environment::with_prefix("tag-temp").create();

    let vars = hashmap! {
        CT_PROJECT => proj.name().as_str(),
        CT_ENVIRONMENT => env.name().as_str()
    };

    cloudtruth!("param set some_param --value 'value first'")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("param set another_param --value 'devops'")
        .envs(&vars)
        .assert()
        .success();

    let test_temp = Name::with_prefix("temp-times");
    let template_text1 = "\
    # just a different template\n\
    references = {{some_param}}\n\
    ";
    let test_file = TestFile::with_contents(template_text1).unwrap();

    cloudtruth!("template set {test_temp} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();

    // create a tag
    cloudtruth!("env tag set '{env}' 'template-tag'")
        .envs(&vars)
        .assert()
        .success();

    // update template and params
    let template_text2 = "\
        # just a different template\n\
        references = {{another_param}}\n\
        ";
    let test_file = TestFile::with_contents(template_text2).unwrap();

    cloudtruth!("template set {test_temp} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("param set some_param --value 'value second'")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("param set another_param --value 'sre'")
        .envs(&vars)
        .assert()
        .success();

    cloudtruth!("template get {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a different template\n\
            references = sre\n\
            ",
        ));

    cloudtruth!("template get --raw {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(template_text2));

    cloudtruth!("template get --as-of 'template-tag' {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a different template\n\
            references = value first\n\
            ",
        ));

    cloudtruth!("template get --raw --as-of 'template-tag' {test_temp}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(template_text1));

    //before project exists
    cloudtruth!("template get --as-of 'my-missing-tag' {test_temp}")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `my-missing-tag` could not be found in environment `{env}`",
        ));

    // check preview
    let preview_template_text = "\
        # just a comment\n\
        this.is.a.template.value={{some_param}}\n\
    ";
    let preview_file = TestFile::with_contents(preview_template_text).unwrap();
    cloudtruth!("template set {test_temp} -b {test_file}")
        .envs(&vars)
        .assert()
        .success();
    cloudtruth!("template preview {preview_file}")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a comment\n\
            this.is.a.template.value=value second\n\
            ",
        ));
    cloudtruth!("template preview {preview_file} --as-of 'template-tag'")
        .envs(&vars)
        .assert()
        .success()
        .stdout(diff(
            "\
            # just a comment\n\
            this.is.a.template.value=value first\n\
            ",
        ));
    //before project exists
    cloudtruth!("template preview {preview_file} --as-of 'my-missing-tag'")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(contains!(
            "Tag `my-missing-tag` could not be found in environment `{env}`",
        ));
}

#[test]
#[use_harness]
fn test_templates_history() {
    let proj = Project::with_prefix("temp-history").create();
    let env = Environment::with_prefix("env-temp-history").create();
    let vars = hashmap! {
        CT_PROJECT => proj.name().as_str(),
        CT_ENVIRONMENT => env.name().as_str()
    };
    // check for no template history at start
    cloudtruth!("template history")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains("No template history in project"));

    // create two templates
    let temp1 = Name::with_prefix("temp1");
    let temp_file1 = TestFile::with_contents("first body").unwrap();
    let temp2 = Name::with_prefix("temp2");
    let temp_file2 = TestFile::with_contents("# bogus text").unwrap();
    cloudtruth!("templates set {temp1} -b {temp_file1} -d 'simple desc'")
        .envs(&vars)
        .assert()
        .success();
    cloudtruth!("templates set {temp2} -b {temp_file2}")
        .envs(&vars)
        .assert()
        .success();
    // get modification timestamp before changes
    let cmd = cloudtruth!("templates list --show-times -f json")
        .envs(&vars)
        .assert()
        .success();
    let modified_at = cmd.get_template_modified_at(1);
    // create a tag before changes
    cloudtruth!("env tag set {env} stable").assert().success();
    // update the templates
    let temp_file1 = TestFile::with_contents("second body").unwrap();
    let temp_file2 = TestFile::with_contents("different temp text").unwrap();
    cloudtruth!("template set {temp1} -b {temp_file1}")
        .envs(&vars)
        .assert()
        .success();
    cloudtruth!("template set {temp2} -b {temp_file2}")
        .envs(&vars)
        .assert()
        .success();
    // get current user
    let user = get_current_user();
    // check all template history
    cloudtruth!("template history -f csv")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains_all!(
            "Date,User,Action,Name,Changes",
            format!(",Service Account ({user}),create,{temp1},"),
            "first body",
            format!(",Service Account ({user}),update,{temp1},"),
            "second body",
            "simple desc",
            format!(",Service Account ({user}),create,{temp2},"),
            "# bogus text",
            format!(",Service Account ({user}),update,{temp2},"),
            "different temp text"
        ));
    // check history of one template
    cloudtruth!("template history '{temp2}' -f csv")
        .envs(&vars)
        .assert()
        .success()
        .stdout(
            not(contains_any!(
                "Date,User,Action,Name,Changes",
                "temp1",
                "first body",
                "second body",
                "simple desc"
            ))
            .and(contains_all!(
                "Date,User,Action,Changes",
                "temp2",
                "# bogus text",
                "different temp text"
            )),
        );
    // check history at timestamp
    cloudtruth!("template history '{temp2}' --as-of '{modified_at}'")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains_all!("temp2", "# bogus text").and(not(contains("different temp text"))));
    // check history at tag
    cloudtruth!("template history '{temp2}' --as-of stable")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains_all!("temp2", "# bogus text").and(not(contains("different temp text"))));
    // delete both templates
    cloudtruth!("templates delete -y '{temp2}'")
        .envs(&vars)
        .assert()
        .success();
    cloudtruth!("templates delete -y '{temp1}'")
        .envs(&vars)
        .assert()
        .success();
    // check that history shows deletion
    cloudtruth!("templates history -f csv")
        .envs(&vars)
        .assert()
        .success()
        .stdout(contains_all!(
            "Date,User,Action,Name,Changes",
            format!(",Service Account ({user}),delete,{temp1},"),
            format!(",Service Account ({user}),delete,{temp2},")
        ));
    // check that we fail to resolve the deleted template
    cloudtruth!("templates history '{temp1}'")
        .envs(&vars)
        .assert()
        .failure()
        .stderr(contains!("No template '{temp1}' found in project '{proj}'"));
}

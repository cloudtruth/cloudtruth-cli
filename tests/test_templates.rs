use cloudtruth_config::{CT_ENVIRONMENT, CT_PROJECT};
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
fn test_template_evaluate_environments() {
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
            template_text
                .replace("{{param1}}", "some val with space")
                .replace("{{secret1}}", "sssshhhhhhh"),
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
            template_text
                .replace("{{param1}}", "some val with space")
                .replace("{{secret1}}", "sssshhhhhhh")
                + "\n",
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
            template_text
                .replace("{{param1}}", "diff_env_value")
                .replace("{{secret1}}", "top-secret"),
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
            template_text
                .replace("{{param1}}", "diff_env_value")
                .replace("{{secret1}}", "top-secret")
                + "\n",
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

use integration_test_harness::prelude::*;

#[integration_test]
fn test_template_basic() {
    let proj = Project::with_prefix("template-proj").create();

    let temp_file = TestFile::with_contents("Text with no params\n").unwrap();

    let temp_name = Name::with_prefix("orig-template");

    cloudtruth!("--project {proj} template ls -v")
        .assert()
        .success()
        .stdout(contains!("No templates in project '{proj}'"));

    // Create template
    cloudtruth!(
        "--project {proj} templates set {temp_name} --desc 'Description on create' --body {temp_file} "
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

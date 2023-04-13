use integration_test_harness::prelude::*;

#[integration_test]
fn test_template_basic() {
    let proj = ScopedProject::with_prefix("template-proj");

    let temp_file = TestFile::with_contents("Text with no params\n").unwrap();

    let temp = TemplateBuilder::with_prefix("orig-template")
        .project(&proj)
        .description("Description on create")
        .body(&temp_file)
        .build();

    cloudtruth!("--project {proj} template ls -v")
        .assert()
        .success()
        .stdout(contains!("No templates in project '{proj}'"));

    // Create/delete the template within scope of this callback
    temp.clone().with_scope(|mut temp| {
        cloudtruth!("--project {proj} template ls -v -f csv")
            .assert()
            .success()
            .stdout(contains!("{temp},Description on create"));

        cloudtruth!("--project {proj} template get {temp}")
            .assert()
            .success()
            .stdout("Text with no params\n");

        cloudtruth!("--project {proj} template validate {temp}")
            .assert()
            .success()
            .stdout(contains("Success"));

        // Update the description
        cloudtruth!("--project {proj} template set {temp} --desc 'Updated description'")
            .assert()
            .success()
            .stdout(contains!("Updated template '{temp}'"));

        cloudtruth!("--project {proj} template ls --values -f csv")
            .assert()
            .success()
            .stdout(contains!("{temp},Updated description"));

        // idempotent - do it again
        cloudtruth!("--project {proj} template set {temp} --desc 'Updated description'")
            .assert()
            .success();

        // rename
        let orig_name = temp.name().clone();
        temp.rename(Name::with_prefix("renamed-temp"));

        cloudtruth!("--project {proj} template ls")
            .assert()
            .success()
            .stdout(contains(&temp).and(not(contains(&orig_name))));

        // attempting to get template that does not exist yield error
        cloudtruth!("--project {proj} template get {orig_name}")
            .assert()
            .failure()
            .stderr(contains!(
                "No template '{orig_name}' found in project '{proj}'"
            ));

        // change the body
        let body = TestFile::with_contents("different fixed value\n").unwrap();

        cloudtruth!("--project {proj} template set {temp} --body '{body}'")
            .assert()
            .success()
            .stdout(contains!("Updated template '{temp}'"));

        cloudtruth!("--project {proj} template get {temp}")
            .assert()
            .success()
            .stdout(diff("different fixed value\n"));

        cloudtruth!("--project {proj} template get {temp} --raw")
            .assert()
            .success()
            .stdout(diff("different fixed value\n"));

        // nothing to update
        cloudtruth!("--project {proj} template set {temp}")
            .assert()
            .success()
            .stderr(contains!(
                "Template '{temp}' not updated: no updated parameters provided"
            ));

        cloudtruth!("--project {proj} templates list")
            .assert()
            .success()
            .stdout(contains(&temp).and(not(contains("Updated description"))));
    });

    // try to delete again
    cloudtruth!("--project {proj} template delete {temp} --confirm")
        .assert()
        .success()
        .stderr(contains!(
            "Template '{temp}' does not exist for project '{proj}'"
        ));
}

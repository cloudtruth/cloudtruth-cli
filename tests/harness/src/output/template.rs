pub trait AssertTemplateExt {
    fn get_template_modified_at(&self, index: usize) -> String;
}

impl AssertTemplateExt for assert_cmd::assert::Assert {
    fn get_template_modified_at(&self, index: usize) -> String {
        let json = serde_json::from_slice::<serde_json::Value>(&self.get_output().stdout)
            .expect("Unable to parse template list JSON");
        let template = json
            .as_object()
            .unwrap()
            .get("template")
            .unwrap()
            .as_array()
            .expect("Expected a JSON array of templates")
            .get(index)
            .expect("Expected at least 1 template, found none");
        template
            .get("Modified At")
            .expect("No property named 'Modified At' found")
            .as_str()
            .expect("Expected 'Modified At' to be a string")
            .to_string()
    }
}

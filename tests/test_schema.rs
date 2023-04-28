use integration_test_harness::prelude::*;

#[test]
#[use_harness]
fn test_schema() {
    let default_cmd = cloudtruth!("schema server").assert().success();
    let default_text = &default_cmd.get_output().stdout;
    let default = serde_yaml::from_slice::<serde_yaml::Value>(default_text).unwrap();

    let yaml_cmd = cloudtruth!("schema server --format yaml")
        .assert()
        .success();
    let yaml_text = &yaml_cmd.get_output().stdout;
    let yaml = serde_yaml::from_slice::<serde_yaml::Value>(yaml_text).unwrap();

    let json_cmd = cloudtruth!("schema server -f json").assert().success();
    let json_text = &json_cmd.get_output().stdout;
    let json = serde_json::from_slice::<serde_json::Value>(json_text).unwrap();

    assert_eq!(default_text, yaml_text);
    assert_eq!(default, yaml);
    assert_eq!(
        default,
        serde_yaml::from_slice::<serde_yaml::Value>(serde_yaml::to_string(&json).unwrap().as_ref())
            .unwrap()
    )
}

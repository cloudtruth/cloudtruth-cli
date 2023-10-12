use cloudtruth_config::CT_PROJECT;
use cloudtruth_test_harness::{
    output::param_types::{ConstraintEntry, ParseParamTypesExt},
    prelude::*,
};
use indoc::formatdoc;

#[test]
#[use_harness]
fn test_types_basic() {
    let param_type = ParamType::with_prefix("type-name").description("Description on create");
    cloudtruth!("types ls -v")
        .assert()
        .success()
        .stdout(not(contains(param_type.name())));
    // create
    let mut param_type = param_type.create();
    cloudtruth!("types ls -f json")
        .assert()
        .success()
        .stdout(json(prop(
            "parameter-type",
            find_entry(
                prop("Name", value(param_type.name())),
                prop("Parent", value("string"))
                    .and(prop("Description", value("Description on create"))),
            ),
        )));
    // update description
    cloudtruth!("types set {param_type} --desc 'Updated description'")
        .assert()
        .success();
    cloudtruth!("types ls -f csv")
        .assert()
        .success()
        .stdout(contains!("{param_type},string,0,Updated description"));
    // idempotent
    cloudtruth!("types set {param_type} --desc 'Updated description'")
        .assert()
        .success();
    param_type.rename(Name::with_prefix("type-rename"));
    cloudtruth!("types list")
        .assert()
        .success()
        .stdout(contains(param_type.name()).and(not(contains("Updated description"))));
    cloudtruth!("types list --show-times -f csv")
        .assert()
        .success()
        .stdout(
            contains("Created At,Modified At")
                .and(contains(param_type.name()).and(contains("Updated description"))),
        );
    //delete
    let deleted_param_type = ParamType::from_name(param_type.name().clone());
    drop(param_type);
    // try to delete again
    cloudtruth!("types delete {deleted_param_type} --confirm")
        .assert()
        .success();
}

#[test]
#[use_harness]
fn test_types_parents() {
    let parent = ParamType::with_prefix("type-parent").create();
    let mid = ParamType::with_prefix("type-mid").parent(&parent).create();
    let child1 = ParamType::with_prefix("type-child1").parent(&mid).create();
    let child2 = ParamType::with_prefix("type-child2").parent(&mid).create();

    cloudtruth!("type ls -v -f csv")
        .assert()
        .success()
        .stdout(contains_all!(
            format!("{parent},string,"),
            format!("{mid},{parent},"),
            format!("{child1},{mid},"),
            format!("{child2},{mid},")
        ));
    cloudtruth!("type tree")
        .assert()
        .success()
        .stdout(contains(formatdoc! {"
          {parent}
              {mid}
                {child1}
                {child2}
        "}));
    cloudtruth!("type delete '{mid}' --confirm")
        .assert()
        .failure()
        .stderr(contains_all!(
            "Cannot remove type because it has children:",
            child1.name().as_str(),
            child2.name().as_str()
        ));
    cloudtruth!("type set '{child2}' --parent '{parent}'")
        .assert()
        .success()
        .stdout(contains!("Updated parameter type '{child2}'"));
    cloudtruth!("parameter-type ls -v -f csv")
        .assert()
        .success()
        .stdout(contains!("{child2},{parent},"));
    cloudtruth!("type set '{child2}' --parent '{mid}' --desc 'My new description'")
        .assert()
        .success();
    cloudtruth!("type ls -f json")
        .assert()
        .success()
        .stdout(json(prop(
            "parameter-type",
            find_entry(
                prop("Name", value(child2.name().as_str())),
                prop("Parent", value(mid.name().as_str()))
                    .and(prop("Description", value("My new description"))),
            ),
        )));
}

#[test]
#[use_harness]
fn test_types_parent_does_not_exist() {
    let parent = Name::with_prefix("missing-parent");
    let child = Name::with_prefix("child");
    cloudtruth!("type set '{child}' --parent '{parent}'")
        .assert()
        .failure()
        .stderr(contains!("No parent parameter type '{parent}' found"));
}

#[test]
#[use_harness]
fn test_types_pagination() {
    const PAGE_SIZE: usize = 5;
    for i in 0..=PAGE_SIZE {
        ParamType::with_prefix(format!("type-page-{i}")).create();
    }
    cloudtruth!("type ls")
        .rest_debug()
        .page_size(PAGE_SIZE)
        .assert()
        .success()
        .paginated(PAGE_SIZE);
}

#[test]
#[use_harness]
fn test_types_integers() {
    let integer_type = ParamType::from_string("integer");
    let parent = ParamType::with_prefix("type-int-parent")
        .parent(&integer_type)
        .create();
    let child = ParamType::with_prefix("type-int-child")
        .parent(&parent)
        .create();
    let proj = Project::with_prefix("type-int").create();
    cloudtruth!("param set param1 -t '{parent}' -v abc")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains("Rule violation: Value is not of type"));
    cloudtruth!("param set param2 -t '{child}' -v abc")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains("Rule violation: Value is not of type"));
    cloudtruth!("type set {child} --min-len -10 --max-len 100 --regex 'abc.*'")
        .assert()
        .failure()
        .stderr(contains_all!(
            "Rule create error",
            format!("max-len rules not valid for {child} parameters"),
            format!("min-len rules not valid for {child} parameters"),
            format!("regex rules not valid for {child} parameters")
        ));
    cloudtruth!("type set {parent} --min -10 --max 100")
        .assert()
        .success();
    cloudtruth!("type ls -f json")
        .assert()
        .success()
        .stdout(json(prop(
            "parameter-type",
            find_entry(
                prop("Name", value(parent.name().as_str())),
                prop("Rules", value("2")),
            )
            .and(find_entry(
                prop("Name", value(child.name().as_str())),
                prop("Rules", value("0")),
            )),
        )));
    cloudtruth!("param set param1 -t '{parent}' -v -11")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is less than the minimum value of ",
        ));
    cloudtruth!("param set param1 -t '{parent}' -v 101")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is greater than the maximum value of ",
        ));
    cloudtruth!("param set param2 -t '{child}' -v -11")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is less than the minimum value of ",
        ));
    cloudtruth!("param set param2 -t '{child}' -v 101")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is greater than the maximum value of ",
        ));
    cloudtruth!("types set {child} --max 90 --min -5")
        .assert()
        .success();
    let rules = cloudtruth!("types ls --rules -f json --show-times")
        .assert()
        .success()
        .parse_param_types_list_with_rules();
    let parent_rules = rules
        .iter()
        .filter(|c| c.name == parent.name().as_str())
        .collect::<Vec<&ConstraintEntry>>();
    assert_eq!(2, parent_rules.len());
    let rule = parent_rules
        .iter()
        .find(|c| c.rule_type == "max")
        .expect("No max constraint found");
    assert_eq!("100", rule.constraint);
    assert_eq!("integer", rule.parent);
    assert_ne!(None, rule.created_at);
    assert_ne!(None, rule.modified_at);
    let rule = parent_rules
        .iter()
        .find(|c| c.rule_type == "min")
        .expect("No min constraint found");
    assert_eq!("-10", rule.constraint);
    assert_eq!("integer", rule.parent);
    assert_ne!(None, rule.created_at);
    assert_ne!(None, rule.modified_at);
    let child_rules = rules
        .iter()
        .filter(|c| c.name == child.name().as_str())
        .collect::<Vec<&ConstraintEntry>>();
    assert_eq!(2, child_rules.len());
    let rule = child_rules
        .iter()
        .find(|c| c.rule_type == "max")
        .expect("No max constraint found");
    assert_eq!("90", rule.constraint);
    assert_eq!(parent.name().as_str(), rule.parent);
    assert_ne!(None, rule.created_at);
    assert_ne!(None, rule.modified_at);
    let rule = child_rules
        .iter()
        .find(|c| c.rule_type == "min")
        .expect("No min constraint found");
    assert_eq!("-5", rule.constraint);
    assert_eq!(parent.name().as_str(), rule.parent);
    assert_ne!(None, rule.created_at);
    assert_ne!(None, rule.modified_at);
    cloudtruth!("type  set {child} --max 101")
        .assert()
        .failure()
        .stderr(contains(
            "Rule update error: Maximum constraint is greater than an existing rule's maximum constraint",
        ));
    cloudtruth!("type set {child} --min -11")
        .assert()
        .failure()
        .stderr(contains(
            "Rule update error: Minimum constraint is less than an existing rule's minimum constraint"
        ));
    cloudtruth!("param set param1 -t '{parent}' -v -11")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is less than the minimum value of ",
        ));
    cloudtruth!("param set param1 -t '{parent}' -v 101")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is greater than the maximum value of ",
        ));
    cloudtruth!("param set param2 -t '{child}' -v -6")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is less than the minimum value of ",
        ));
    cloudtruth!("param set param2 -t '{child}' -v 91")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is greater than the maximum value of ",
        ));
    cloudtruth!("param set param1 -t '{child}' -v -11")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is less than the minimum value of ",
        ));
    cloudtruth!("param set param1 -t '{child}' -v 101")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is greater than the maximum value of ",
        ));
    cloudtruth!("param set param1 -t '{parent}' -v 45")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success();
    cloudtruth!("param set param2 -t '{child}' -v 46")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success();
    cloudtruth!("param get param1")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success()
        .stdout(contains("45"));
    cloudtruth!("param get param2")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success()
        .stdout(contains("46"));
    cloudtruth!("type set {child} --max 80 --min 5")
        .assert()
        .success();
    cloudtruth!("type ls --rules -f csv")
        .assert()
        .success()
        .stdout(contains!("{child},{parent},max,80").and(contains!("{child},{parent},min,5")));
    cloudtruth!("type set {parent} --max 44")
        .assert()
        .failure()
        .stderr(contains!(
            "Rule update error: Rule may not be applied to {parent}: param1"
        ));
    cloudtruth!("type set {parent} --min 46")
        .assert()
        .failure()
        .stderr(contains!(
            "Rule update error: Rule may not be applied to {parent}: param1"
        ));
    cloudtruth!("type set {child} --max 45")
        .assert()
        .failure()
        .stderr(contains!(
            "Rule update error: Rule may not be applied to {child}: param2"
        ));
    cloudtruth!("type set {child} --min 47")
        .assert()
        .failure()
        .stderr(contains!(
            "Rule update error: Rule may not be applied to {child}: param2"
        ));
    cloudtruth!("type set {parent} --no-min --no-max")
        .assert()
        .success();
    cloudtruth!("ls -f csv")
        .assert()
        .success()
        .stdout(contains!("{parent},integer,0,").and(contains!("{child},{parent},2,")));
    cloudtruth!("param set param1 -v 110")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success();
    cloudtruth!("param get param1")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success()
        .stdout(contains("110"));
    cloudtruth!("param set param2 -v 110")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .failure()
        .stderr(contains(
            "Rule violation: Value is greater than the maximum value of ",
        ));
    cloudtruth!("param set {parent} --no-min --no-max")
        .env(CT_PROJECT, proj.name().as_str())
        .assert()
        .success();
}

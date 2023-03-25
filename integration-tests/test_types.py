from testcase import TestCase
from testcase import find_by_prop
from testcase import TEST_PAGE_SIZE
from testcase import PROP_NAME
from testcase import PROP_CREATED
from testcase import PROP_MODIFIED
from testcase import skip_known_issue

PROP_CONSTRAINT = "Constraint"
PROP_COUNT = "Rules"
PROP_PARENT = "Parent"
PROP_TYPE = "Rule Type"


class TestParameterTypes(TestCase):
    def test_type_basic(self):
        # verify `type_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        type_name = self.make_name("type-name")
        sub_cmd = base_cmd + "types "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(type_name, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f'set {type_name} --desc "{orig_desc}"')
        self.assertResultSuccess(result)
        entries = self.get_cli_entries(cmd_env, sub_cmd + "ls -f json", "parameter-type")
        entry = find_by_prop(entries, PROP_NAME, type_name)[0]
        self.assertEqual(entry.get("Parent"), "string")  # default
        self.assertEqual(entry.get("Description"), orig_desc)

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f'set {type_name} --desc "{new_desc}"')
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name},string,0,{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f'set {type_name} --desc "{new_desc}"')
        self.assertResultSuccess(result)

        # rename
        orig_name = type_name
        type_name = self.make_name("type-rename")
        result = self.run_cli(cmd_env, sub_cmd + f'set {orig_name} --rename "{type_name}"')
        self.assertResultSuccess(result)
        self.assertIn(f"Updated parameter type '{type_name}'", result.out())

        # test the list without the values
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(type_name, result.out())
        self.assertNotIn(new_desc, result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At", result.out())
        self.assertIn(type_name, result.out())
        self.assertIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {type_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(type_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {type_name} --confirm")
        self.assertResultWarning(result, f"Parameter type '{type_name}' does not exist")

    @skip_known_issue("SC-9666")
    def test_type_parents(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        type_name1 = self.make_name("type-par-1")
        type_name2 = self.make_name("type-mid-2")
        type_name3 = self.make_name("type-chld-3")
        type_name4 = self.make_name("type-chld-4")

        self.create_type(cmd_env, type_name1)
        self.create_type(cmd_env, type_name2, parent=type_name1)
        self.create_type(cmd_env, type_name3, parent=type_name2)
        self.create_type(cmd_env, type_name4, parent=type_name2)

        # Use csv to validate, since the names may be variable
        result = self.run_cli(cmd_env, base_cmd + "type ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name1},string,", result.out())
        self.assertIn(f"{type_name2},{type_name1},", result.out())
        self.assertIn(f"{type_name3},{type_name2},", result.out())
        self.assertIn(f"{type_name4},{type_name2},", result.out())

        # basic 'tree' test
        result = self.run_cli(cmd_env, base_cmd + "ty tree")
        self.assertResultSuccess(result)
        expected = f"  {type_name1}\n    {type_name2}\n      {type_name3}\n      {type_name4}\n"
        self.assertIn(expected, result.out())

        # attempt to delete something that is used elsewhere
        result = self.run_cli(cmd_env, base_cmd + f"type delete '{type_name2}' --confirm")
        self.assertResultError(result, "Cannot remove type because it has children: ")
        self.assertIn(type_name3, result.err())
        self.assertIn(type_name4, result.err())

        # attempt to create without an existing parent
        type_name5 = self.make_name("type-par-5")
        type_name6 = self.make_name("type-par-6")
        result = self.run_cli(cmd_env, base_cmd + f"type set '{type_name5}' --parent '{type_name6}'")
        self.assertResultError(result, f"No parent parameter type '{type_name6}' found")

        # update parent -- success
        result = self.run_cli(cmd_env, base_cmd + f"type set '{type_name4}' --parent '{type_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated parameter type '{type_name4}'", result.out())

        result = self.run_cli(cmd_env, base_cmd + "parameter-type ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name4},{type_name1},", result.out())

        # setting to same parent is ignored
        new_desc = "My new description"
        cmd = base_cmd + f"type set '{type_name4}' --parent '{type_name2}' --desc '{new_desc}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        # make sure description was updated, yet parent remains
        entries = self.get_cli_entries(cmd_env, base_cmd + "ty ls -f json", "parameter-type")
        entry = find_by_prop(entries, PROP_NAME, type_name4)[0]
        self.assertEqual(entry.get("Parent"), type_name2)
        self.assertEqual(entry.get("Description"), new_desc)

        # cleanup - unwind the stack
        self.delete_type(cmd_env, type_name4)
        self.delete_type(cmd_env, type_name3)
        self.delete_type(cmd_env, type_name2)
        self.delete_type(cmd_env, type_name1)

    def test_type_pagination(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        page_size = TEST_PAGE_SIZE
        type_count = page_size + 1

        for idx in range(type_count):
            type_name = self.make_name(f"pag-{idx}")
            self.create_type(cmd_env, type_name)

        self.assertPaginated(cmd_env, base_cmd + "type ls", "/types/?")

        # cleanup
        for idx in range(type_count):
            type_name = self.make_name(f"pag-{idx}")
            self.delete_type(cmd_env, type_name)

    def test_type_integers(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        type_cmd = base_cmd + "type "

        # create a couple types based off an integer
        base_type = "integer"
        type_name1 = self.make_name("type-int-parent")
        self.create_type(cmd_env, type_name1, parent=base_type)
        type_name2 = self.make_name("type-int-child")
        self.create_type(cmd_env, type_name2, parent=type_name1)

        ###################
        # create project/parameters to use
        invalid_type_err = "Rule violation: Value is not of type"
        proj_name = self.make_name("type-int")
        self.create_project(cmd_env, proj_name)
        param1 = "param1"
        param2 = "param2"
        param_cmd = base_cmd + f"--project '{proj_name}' param "
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v abc")
        self.assertResultError(result, invalid_type_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v abc")
        self.assertResultError(result, invalid_type_err)

        ###################
        # cannot set some rules on integer types
        min_len = -10
        max_len = 100
        regex = "abc.*"
        cmd = type_cmd + f"set {type_name2} --min-len {min_len} --max-len {max_len} --regex '{regex}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, "Rule create error")
        self.assertIn(f"max-len rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"min-len rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"regex rules not valid for {type_name2} parameters", result.err())

        ###################
        # add some rules
        min_a = -10
        max_a = 100
        cmd = type_cmd + f"set {type_name1} --max {max_a} --min {min_a}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        entries = self.get_cli_entries(cmd_env, type_cmd + "ls -f json", "parameter-type")
        entry = find_by_prop(entries, PROP_NAME, type_name1)[0]
        self.assertEqual(entry.get(PROP_COUNT), "2")
        entry = find_by_prop(entries, PROP_NAME, type_name2)[0]
        self.assertEqual(entry.get(PROP_COUNT), "0")

        ###################
        # parameter range checking, before child rules
        less_than_err = "Rule violation: Value is less than the minimum value of "
        greater_than_err = "Rule violation: Value is greater than the maximum value of "
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {min_a - 1}")
        self.assertResultError(result, less_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {max_a + 1}")
        self.assertResultError(result, greater_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {min_a - 1}")
        self.assertResultError(result, less_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {max_a + 1}")
        self.assertResultError(result, greater_than_err)

        ###################
        # add child rules
        min_b = -5
        max_b = 90
        cmd = type_cmd + f"set {type_name2} --max {max_b} --min {min_b}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        entries = self.get_cli_entries(cmd_env, type_cmd + "ls --rules -f json --show-times", "parameter-type")
        entry1 = find_by_prop(entries, PROP_NAME, type_name1)
        self.assertEqual(len(entry1), 2)
        entry = find_by_prop(entry1, PROP_TYPE, "max")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(max_a))
        self.assertEqual(entry.get(PROP_PARENT), base_type)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entry1, PROP_TYPE, "min")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(min_a))
        self.assertEqual(entry.get(PROP_PARENT), base_type)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry2 = find_by_prop(entries, PROP_NAME, type_name2)
        self.assertEqual(len(entry2), 2)
        entry = find_by_prop(entry2, PROP_TYPE, "max")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(max_b))
        self.assertEqual(entry.get(PROP_PARENT), type_name1)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entry2, PROP_TYPE, "min")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(min_b))
        self.assertEqual(entry.get(PROP_PARENT), type_name1)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))

        # invalid constraints -- child cannot be more permissive than parent
        err_msg = "Rule update error: Maximum constraint is greater than an existing rule's maximum constraint"
        cmd = type_cmd + f"set {type_name2} --max {max_a + 1}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, err_msg)
        err_msg = "Rule update error: Minimum constraint is less than an existing rule's minimum constraint"
        cmd = type_cmd + f"set {type_name2} --min {min_a - 1}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, err_msg)

        ###################
        # parameter range checking, after child rules
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {min_a - 1}")
        self.assertResultError(result, less_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {max_a + 1}")
        self.assertResultError(result, greater_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {min_b - 1}")
        self.assertResultError(result, less_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {max_b + 1}")
        self.assertResultError(result, greater_than_err)
        # must meet parent constraints, too
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name2}' -v {min_a - 1}")
        self.assertResultError(result, less_than_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name2}' -v {max_a + 1}")
        self.assertResultError(result, greater_than_err)

        # check success
        value_a = int(min_a + (max_a - min_a) / 2)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {value_a}")
        self.assertResultSuccess(result)
        value_b = value_a + 1
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {value_b}")
        self.assertResultSuccess(result)
        self.verify_param(cmd_env, proj_name, param1, str(value_a))
        self.verify_param(cmd_env, proj_name, param2, str(value_b))

        ###################
        # update child rules to be more restrictive
        min_b = value_b - 10
        max_b = value_b + 10
        cmd = type_cmd + f"set {type_name2} --max {max_b} --min {min_b}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, type_cmd + "ls --rules -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name2},{type_name1},max,{max_b}", result.out())
        self.assertIn(f"{type_name2},{type_name1},min,{min_b}", result.out())

        ###################
        # fail to change rules with type in use and out of bounds
        def update_err(type_name: str, param_name: str) -> str:
            return f"Rule update error: Rule may not be applied to {type_name}: {param_name}"

        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --max {value_a - 1}")
        self.assertResultError(result, update_err(type_name1, param1))
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --min {value_a + 1}")
        self.assertResultError(result, update_err(type_name1, param1))
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name2} --max {value_b - 1}")
        self.assertResultError(result, update_err(type_name2, param2))
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name2} --min {value_b + 1}")
        self.assertResultError(result, update_err(type_name2, param2))

        ###################
        # delete rules - starting with parent rules
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --no-min --no-max")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, type_cmd + "ls -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name1},{base_type},0,", result.out())
        self.assertIn(f"{type_name2},{type_name1},2,", result.out())

        # see the constraint has gone away on parent, but not child
        new_value = str(max_a + 10)
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -v {new_value}")
        self.assertResultSuccess(result)
        self.verify_param(cmd_env, proj_name, param1, new_value)
        result = self.run_cli(cmd_env, param_cmd + f"set {param2} -v {new_value}")
        self.assertResultError(result, greater_than_err)

        # idempotent
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --no-min --no-max")
        self.assertResultSuccess(result)

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_type(cmd_env, type_name2)
        self.delete_type(cmd_env, type_name1)

    @skip_known_issue("SC-9666")
    def test_type_boolean(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        type_cmd = base_cmd + "type "

        # create a couple types based off an integer
        base_type = "boolean"
        type_name1 = self.make_name("type-bool-parent")
        self.create_type(cmd_env, type_name1, parent=base_type)
        type_name2 = self.make_name("type-bool-child")
        self.create_type(cmd_env, type_name2, parent=type_name1)

        ###################
        # create project/parameters to use
        invalid_type_err = "Rule violation: Value is not of type"
        proj_name = self.make_name("type-bool")
        self.create_project(cmd_env, proj_name)
        param1 = "param1"
        param2 = "param2"
        param_cmd = base_cmd + f"--project '{proj_name}' param "
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v abc")
        self.assertResultError(result, invalid_type_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v abc")
        self.assertResultError(result, invalid_type_err)

        ###################
        # cannot set some rules on integer types
        min_len = -10
        max_len = 100
        regex = "abc.*"
        min_value = -100
        max_value = 1000
        cmd = (
            type_cmd
            + f"set {type_name2} --min-len {min_len} --max-len {max_len} "
            + f"--regex '{regex}' --min {min_value} --max {max_value}"
        )
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, "Rule create error")
        self.assertIn(f"max-len rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"min-len rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"regex rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"min rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"max rules not valid for {type_name2} parameters", result.err())

        ###################
        # no rules were/can be added
        entries = self.get_cli_entries(cmd_env, type_cmd + "ls -v -f json --show-times --rules", "parameter-type")
        self.assertEqual(0, len(find_by_prop(entries, PROP_NAME, type_name1)))
        self.assertEqual(0, len(find_by_prop(entries, PROP_NAME, type_name2)))

        entries = self.get_cli_entries(cmd_env, type_cmd + "ls -f json --show-times", "parameter-type")
        entry = find_by_prop(entries, PROP_NAME, type_name1)[0]
        self.assertEqual(entry.get(PROP_PARENT), base_type)
        self.assertEqual(entry.get(PROP_COUNT), str(0))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entries, PROP_NAME, type_name2)[0]
        self.assertEqual(entry.get(PROP_PARENT), type_name1)
        self.assertEqual(entry.get(PROP_COUNT), str(0))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))

        ###################
        # more parameter range checking
        int_value = 22  # NOTE: 1 and 0 are considered valid booleans
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {int_value}")
        self.assertResultError(result, invalid_type_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {int_value}")
        self.assertResultError(result, invalid_type_err)

        # check success
        value_a = "true"
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {value_a}")
        self.assertResultSuccess(result)
        value_b = "false"
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {value_b}")
        self.assertResultSuccess(result)
        self.verify_param(cmd_env, proj_name, param1, value_a)
        self.verify_param(cmd_env, proj_name, param2, value_b)

        ###################
        # delete rules - starting with parent rules
        del_rules_cmd = type_cmd + f"set {type_name1} --no-min --no-max --no-min-len --no-max-len --no-regex"
        result = self.run_cli(cmd_env, del_rules_cmd)
        self.assertResultSuccess(result)

        # idempotent
        result = self.run_cli(cmd_env, del_rules_cmd)
        self.assertResultSuccess(result)

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_type(cmd_env, type_name2)
        self.delete_type(cmd_env, type_name1)

    def test_type_strings(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        type_cmd = base_cmd + "type "

        # create a couple types based off an integer
        base_type = "string"
        type_name1 = self.make_name("type-str-parent")
        self.create_type(cmd_env, type_name1)  # unspecified defaults to string
        type_name2 = self.make_name("type-str-child")
        self.create_type(cmd_env, type_name2, parent=type_name1)

        ###################
        # create project/parameters to use
        proj_name = self.make_name("type-str")
        self.create_project(cmd_env, proj_name)
        param1 = "param1"
        param2 = "param2"
        param_cmd = base_cmd + f"--project '{proj_name}' param "

        ###################
        # cannot set some rules on integer types
        min_value = -10
        max_value = 100
        cmd = type_cmd + f"set {type_name2} --min {min_value} --max {max_value}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, "Rule create error")
        self.assertIn(f"max rules not valid for {type_name2} parameters", result.err())
        self.assertIn(f"min rules not valid for {type_name2} parameters", result.err())

        ###################
        # add some rules
        min_len_a = 5
        max_len_a = 20
        regex_a = "a.*"  # starts with 'a'
        cmd = type_cmd + f"set {type_name1} --max-len {max_len_a} --min-len {min_len_a} --regex '{regex_a}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        entries = self.get_cli_entries(cmd_env, type_cmd + "ls -f json", "parameter-type")
        entry = find_by_prop(entries, PROP_NAME, type_name1)[0]
        self.assertEqual(entry.get(PROP_COUNT), "3")
        entry = find_by_prop(entries, PROP_NAME, type_name2)[0]
        self.assertEqual(entry.get(PROP_COUNT), "0")

        ###################
        # parameter range checking, before child rules
        def V(length: int, char: str = "a") -> str:
            return char * length

        too_short_err = "Rule violation: Value must be at least "
        too_long_err = "Rule violation: Value must be at most "
        regex_err = "Rule violation: Value does not match regular expression "
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(min_len_a - 1)}")
        self.assertResultError(result, too_short_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(max_len_a + 1)}")
        self.assertResultError(result, too_long_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(min_len_a, 'b')}")
        self.assertResultError(result, regex_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {V(min_len_a - 1)}")
        self.assertResultError(result, too_short_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {V(max_len_a + 1)}")
        self.assertResultError(result, too_long_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(min_len_a, 'b')}")
        self.assertResultError(result, regex_err)

        ###################
        # add child rules
        min_len_b = 6  # would like to do 1, but that means no string is provided...
        max_len_b = 19
        regex_b = ".*b"  # ends with 'b'
        cmd = type_cmd + f"set {type_name2} --max-len {max_len_b} --min-len {min_len_b} --regex '{regex_b}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        entries = self.get_cli_entries(cmd_env, type_cmd + "ls --rules -f json --show-times", "parameter-type")
        entry1 = find_by_prop(entries, PROP_NAME, type_name1)
        self.assertEqual(len(entry1), 3)
        entry = find_by_prop(entry1, PROP_TYPE, "max-len")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(max_len_a))
        self.assertEqual(entry.get(PROP_PARENT), base_type)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entry1, PROP_TYPE, "min-len")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(min_len_a))
        self.assertEqual(entry.get(PROP_PARENT), base_type)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entry1, PROP_TYPE, "regex")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), regex_a)
        self.assertEqual(entry.get(PROP_PARENT), base_type)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry2 = find_by_prop(entries, PROP_NAME, type_name2)
        self.assertEqual(len(entry2), 3)
        entry = find_by_prop(entry2, PROP_TYPE, "max-len")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(max_len_b))
        self.assertEqual(entry.get(PROP_PARENT), type_name1)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entry2, PROP_TYPE, "min-len")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), str(min_len_b))
        self.assertEqual(entry.get(PROP_PARENT), type_name1)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        entry = find_by_prop(entry2, PROP_TYPE, "regex")[0]
        self.assertEqual(entry.get(PROP_CONSTRAINT), regex_b)
        self.assertEqual(entry.get(PROP_PARENT), type_name1)
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))

        # child rules must be more restrictive
        err_msg = (
            "Rule update error: Maximum length constraint is greater than an existing rule's maximum length constraint"
        )
        cmd = type_cmd + f"set {type_name2} --max-len {max_len_a + 1}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, err_msg)
        err_msg = (
            "Rule update error: Minimum length constraint is less than an existing rule's minimum length constraint"
        )
        cmd = type_cmd + f"set {type_name2} --min-len {min_len_a - 1}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, err_msg)

        ###################
        # parameter range checking, after child rules
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(min_len_a - 1)}")
        self.assertResultError(result, too_short_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(max_len_a + 1)}")
        self.assertResultError(result, too_long_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {V(max_len_a, 'b')}")
        self.assertResultError(result, regex_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {V(min_len_b - 1)}")
        self.assertResultError(result, too_short_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {V(max_len_b + 1)}")
        self.assertResultError(result, too_long_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {V(max_len_a - 1, 'c')}")
        self.assertResultError(result, regex_err)
        # must meet parent constraints, too
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name2}' -v {V(min_len_a - 1)}")
        self.assertResultError(result, too_short_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name2}' -v {V(max_len_a + 1)}")
        self.assertResultError(result, too_long_err)
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name2}' -v {V(max_len_a, 'b')}")
        self.assertResultError(result, regex_err)

        # check success
        value_a = V(int(min_len_a + (max_len_a - min_len_a) / 2))
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -t '{type_name1}' -v {value_a}")
        self.assertResultSuccess(result)
        value_b = V(len(value_a)) + "b"
        result = self.run_cli(cmd_env, param_cmd + f"set '{param2}' -t '{type_name2}' -v {value_b}")
        self.assertResultSuccess(result)
        self.verify_param(cmd_env, proj_name, param1, str(value_a))
        self.verify_param(cmd_env, proj_name, param2, str(value_b))

        ###################
        # update child rules to be more restrictive
        min_len_b = len(value_b) - 3
        max_len_b = len(value_b) + 3
        cmd = type_cmd + f"set {type_name2} --max-len {max_len_b} --min-len {min_len_b}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, type_cmd + "ls --rules -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name2},{type_name1},max-len,{max_len_b}", result.out())
        self.assertIn(f"{type_name2},{type_name1},min-len,{min_len_b}", result.out())

        ###################
        # fail to change rules with type in use and out of bounds
        def update_err(type_name: str, param_name: str) -> str:
            return f"Rule update error: Rule may not be applied to {type_name}: {param_name}"

        curr_len = len(value_a)
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --max-len {curr_len - 1}")
        self.assertResultError(result, update_err(type_name1, param1))
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --min-len {curr_len + 1}")
        self.assertResultError(result, update_err(type_name1, param1))
        curr_len = len(value_b)
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name2} --max-len {curr_len - 1}")
        self.assertResultError(result, update_err(type_name2, param2))
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name2} --min-len {curr_len + 1}")
        self.assertResultError(result, update_err(type_name2, param2))

        ###################
        # delete rules - starting with parent rules
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --no-min-len --no-max-len --no-regex")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, type_cmd + "ls -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name1},{base_type},0,", result.out())
        self.assertIn(f"{type_name2},{type_name1},3,", result.out())

        # see the constraint has gone away on parent, but not child
        new_value = V(max_len_a + 10, "b")
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -v {new_value}")
        self.assertResultSuccess(result)
        self.verify_param(cmd_env, proj_name, param1, new_value)
        result = self.run_cli(cmd_env, param_cmd + f"set {param2} -v {new_value}")
        self.assertResultError(result, too_long_err)

        # idempotent
        result = self.run_cli(cmd_env, type_cmd + f"set {type_name1} --no-min-len --no-max-len --no-regex")
        self.assertResultSuccess(result)

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_type(cmd_env, type_name2)
        self.delete_type(cmd_env, type_name1)

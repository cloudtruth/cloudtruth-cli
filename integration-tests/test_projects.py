from testcase import TestCase
from testcase import find_by_prop
from testcase import TEST_PAGE_SIZE
from testcase import skip_known_issue


class TestProjects(TestCase):
    def test_project_basic(self):
        # verify `proj_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("proj-name")
        sub_cmd = base_cmd + "projects "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(proj_name, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f'set {proj_name} --desc "{orig_desc}"')
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name},,{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f'set {proj_name} --desc "{new_desc}"')
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name},,{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f'set {proj_name} --desc "{new_desc}"')
        self.assertResultSuccess(result)

        # rename
        orig_name = proj_name
        proj_name = self.make_name("proj-rename")
        result = self.run_cli(cmd_env, sub_cmd + f'set {orig_name} --rename "{proj_name}"')
        self.assertResultSuccess(result)
        self.assertIn(f"Updated project '{proj_name}'", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name}")
        self.assertResultWarning(
            result,
            f"Project '{proj_name}' not updated: no updated parameters provided",
        )

        # test the list without the values
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(proj_name, result.out())
        self.assertNotIn(new_desc, result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At", result.out())
        self.assertIn(proj_name, result.out())
        self.assertIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(proj_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertResultWarning(result, f"Project '{proj_name}' does not exist")

    def test_project_parents(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name1 = self.make_name("proj-par-1")
        proj_name2 = self.make_name("proj-mid-2")
        proj_name3 = self.make_name("proj-chld-3")
        proj_name4 = self.make_name("proj-chld-4")

        self.create_project(cmd_env, proj_name1)
        self.create_project(cmd_env, proj_name2, parent=proj_name1)
        self.create_project(cmd_env, proj_name3, parent=proj_name2)
        self.create_project(cmd_env, proj_name4, parent=proj_name2)

        # Use csv to validate, since the names may be variable
        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name1},,", result.out())
        self.assertIn(f"{proj_name2},{proj_name1},", result.out())
        self.assertIn(f"{proj_name3},{proj_name2},", result.out())
        self.assertIn(f"{proj_name4},{proj_name2},", result.out())

        # basic 'tree' test
        result = self.run_cli(cmd_env, base_cmd + "proj tree")
        self.assertResultSuccess(result)
        expected = f"{proj_name1}\n  {proj_name2}\n    {proj_name3}\n    {proj_name4}\n"
        self.assertIn(expected, result.out())

        # attempt to delete something that is used elsewhere
        result = self.run_cli(cmd_env, base_cmd + f"project delete '{proj_name2}' --confirm")
        self.assertResultError(result, f"Cannot delete {proj_name2} because the following projects depend on it")
        self.assertIn(proj_name3, result.err())
        self.assertIn(proj_name4, result.err())

        # attempt to create without an existing parent
        proj_name5 = self.make_name("proj-par-5")
        proj_name6 = self.make_name("proj-par-6")
        result = self.run_cli(cmd_env, base_cmd + f"project set '{proj_name5}' --parent '{proj_name6}'")
        self.assertResultError(result, f"No parent project '{proj_name6}' found")

        # update parent -- success
        result = self.run_cli(cmd_env, base_cmd + f"project set '{proj_name4}' --parent '{proj_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated project '{proj_name4}'", result.out())

        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name4},{proj_name1},", result.out())

        # setting to same parent is ignored
        new_desc = "My new description"
        cmd = base_cmd + f"project set '{proj_name4}' --parent '{proj_name2}' --desc '{new_desc}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        # make sure description was updated, yet parent remains
        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name4},{proj_name2},{new_desc}", result.out())

        # cleanup - unwind the stack
        self.delete_project(cmd_env, proj_name4)
        self.delete_project(cmd_env, proj_name3)
        self.delete_project(cmd_env, proj_name2)
        self.delete_project(cmd_env, proj_name1)

    @skip_known_issue("SC-9178")
    def test_project_parent_parameters(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name1 = self.make_name("ppp-parent")
        proj_name2 = self.make_name("ppp-child")
        self.create_project(cmd_env, proj_name1)
        self.create_project(cmd_env, proj_name2)
        proj1_cmd = base_cmd + f"--project '{proj_name1}' param "
        proj2_cmd = base_cmd + f"--project '{proj_name2}' param "

        param1 = "param1"
        type1 = "integer"
        type2 = "boolean"
        value1 = "1"
        value2 = "false"
        self.set_param(cmd_env, proj_name1, param1, value1, param_type=type1)
        self.set_param(cmd_env, proj_name2, param1, value2, param_type=type2)

        # able to set parent, even though overlapped  parameter names
        result = self.run_cli(cmd_env, base_cmd + f"proj set '{proj_name2}' -p '{proj_name1}'")
        self.assertResultSuccess(result)

        # verify relationship is established
        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name2},{proj_name1},", result.out())

        # verify values still same for different projects
        result = self.run_cli(cmd_env, proj1_cmd + f"get '{param1}' -d")
        self.assertResultSuccess(result)
        self.assertIn(f"Parameter Type: {type1}", result.out())
        self.assertIn(f"Value: {value1}", result.out())

        result = self.run_cli(cmd_env, proj2_cmd + f"get '{param1}' -d")
        self.assertResultSuccess(result)
        self.assertIn(f"Parameter Type: {type2}", result.out())
        self.assertIn(f"Value: {value2}", result.out())

        self.delete_param(cmd_env, proj_name1, param1)
        self.delete_param(cmd_env, proj_name2, param1)

        # now, try to add a value to the parent
        result = self.run_cli(cmd_env, proj1_cmd + f"set '{param1}' -v '{value1}' -t {type1} --min 0 --max 10")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, proj1_cmd + f"get '{param1}' -d")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value1}", result.out())

        # since it is in the parent, it is visible in both parent and child
        self.verify_param(cmd_env, proj_name1, param1, value1)
        self.verify_param(cmd_env, proj_name2, param1, value1)

        # verify it cannot be set from the child without the special flag
        value2 = "8"
        cmd = base_cmd + f"--project '{proj_name2}' param set '{param1}' -v '{value2}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, f"Parameter '{param1}' must be set from project '{proj_name1}'")

        # value is unchanged
        self.verify_param(cmd_env, proj_name1, param1, value1)
        self.verify_param(cmd_env, proj_name2, param1, value1)

        # do it again with the special flag
        result = self.run_cli(cmd_env, cmd + "--create-child")
        self.assertResultSuccess(result)

        # check the rules/types/values
        result = self.run_cli(cmd_env, proj1_cmd + f"get -d {param1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn(f"Parameter Type: {type1}", result.out())
        self.assertIn("Rule Count: 2", result.out())

        # inherit types/rules
        result = self.run_cli(cmd_env, proj2_cmd + f"get -d {param1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value2}", result.out())
        self.assertIn(f"Parameter Type: {type1}", result.out())
        self.assertIn("Rule Count: 2", result.out())

        # now let parent/child be different types... really should be able to do this in 1 command
        value2 = "true"
        cmd = proj2_cmd + f"set '{param1}' --no-max --no-min"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        cmd = proj2_cmd + f"set '{param1}' -v '{value2}' -t string"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        cmd = proj2_cmd + f"set '{param1}' -t '{type2}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        # check the rules/types/values
        result = self.run_cli(cmd_env, proj1_cmd + f"get -d {param1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn(f"Parameter Type: {type1}", result.out())
        self.assertIn("Rule Count: 2", result.out())

        # inherit types/rules
        result = self.run_cli(cmd_env, proj2_cmd + f"get -d {param1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value2}", result.out())
        self.assertIn(f"Parameter Type: {type2}", result.out())
        self.assertIn("Rule Count: 0", result.out())

        # remove the environmental override
        result = self.run_cli(cmd_env, proj2_cmd + f"unset '{param1}'")
        self.assertResultSuccess(result)

        # now, we get a non-boolean type back from the parent
        result = self.run_cli(cmd_env, proj2_cmd + f"get -d {param1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn(f"Parameter Type: {type2}", result.out())

        # remove the parent relationship
        result = self.run_cli(cmd_env, base_cmd + f"proj set '{proj_name2}' --parent ''")
        self.assertResultSuccess(result)

        # verify parent is remove
        projects = self.get_cli_entries(cmd_env, base_cmd + "proj ls -v -f json", "project")
        entry = find_by_prop(projects, "Name", proj_name2)[0]
        self.assertEqual(entry.get("Parent"), "")

        # see it we still have it
        result = self.list_params(cmd_env, proj_name2, show_values=False)
        self.assertIn(param1, result.out())

        # cleanup -- need to unwind in order
        self.delete_project(cmd_env, proj_name2)
        self.delete_project(cmd_env, proj_name1)

    def test_project_pagination(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        page_size = TEST_PAGE_SIZE
        proj_count = page_size + 1

        for idx in range(proj_count):
            proj_name = self.make_name(f"pag-{idx}")
            self.create_project(cmd_env, proj_name)

        self.assertPaginated(cmd_env, base_cmd + "proj ls", "/projects/?")

        # cleanup
        for idx in range(proj_count):
            proj_name = self.make_name(f"pag-{idx}")
            self.delete_project(cmd_env, proj_name)

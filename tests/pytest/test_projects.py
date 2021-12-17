import unittest

from testcase import TestCase
from testcase import find_by_prop
from testcase import TEST_PAGE_SIZE


class TestProjects(TestCase):
    def test_project_basic(self):
        # verify `proj_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-proj-name")
        sub_cmd = base_cmd + "projects "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(proj_name, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{orig_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name},,{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name},,{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        # rename
        orig_name = proj_name
        proj_name = self.make_name("test-proj-rename")
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{proj_name}\"")
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

    @unittest.skip("Update test for changed behavior")
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
        self.assertResultError(result, "Cannot remove project because the following project(s) depend on it")
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

        ###########
        # cannot change to parent with same variables
        self.create_project(cmd_env, proj_name5)
        self.create_project(cmd_env, proj_name6)

        param1 = "param1"
        value1 = "this"
        self.set_param(cmd_env, proj_name5, param1, value1)
        self.set_param(cmd_env, proj_name6, param1, value1)

        # see fail to set parent
        conflict_msg = "Parameter(s) would not be unique when including project dependencies"
        result = self.run_cli(cmd_env, base_cmd + f"proj set '{proj_name6}' -p '{proj_name5}'")
        self.assertResultError(result, conflict_msg)
        self.assertIn(param1, result.err())

        # verify relationship not established
        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name6},,", result.out())  # no parent listed

        ###########
        # test adding duplicate parameters to parent/child after relationship is esstablished

        # setup the relationship
        self.delete_param(cmd_env, proj_name5, param1)
        result = self.run_cli(cmd_env, base_cmd + f"proj set '{proj_name6}' -p '{proj_name5}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name6},{proj_name5},", result.out())

        # now, try to add a value to the parent
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj_name5}' param set '{param1}' -v '{value1}'")
        self.assertResultError(result, conflict_msg)
        self.assertIn(param1, result.err())

        # verify not added
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj_name5}' param ls")
        self.assertResultSuccess(result)
        self.assertNotIn(param1, result.out())

        # repeat adding to child
        self.delete_param(cmd_env, proj_name6, param1)
        self.set_param(cmd_env, proj_name5, param1, value1)

        # since it is in the parent, it is visible in both parent and child
        self.verify_param(cmd_env, proj_name5, param1, value1)
        self.verify_param(cmd_env, proj_name6, param1, value1)

        # verify it cannot be set from the child
        value2 = "different value"
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj_name6}' param set '{param1}' -v '{value2}'")
        self.assertResultError(result, f"Parameter '{param1}' must be set from project '{proj_name5}'")

        # value is unchanged
        self.verify_param(cmd_env, proj_name5, param1, value1)
        self.verify_param(cmd_env, proj_name6, param1, value1)

        # remove the parent relationship
        result = self.run_cli(cmd_env, base_cmd + f"proj set '{proj_name6}' --parent ''")
        self.assertResultSuccess(result)

        # verify parent is remove
        result = self.run_cli(cmd_env, base_cmd + "proj ls -v -f json")
        self.assertResultSuccess(result)
        projects = eval(result.out()).get("project")
        entry = find_by_prop(projects, "Name", proj_name6)[0]
        self.assertEqual(entry.get("Parent"), "")

        # see it no longer has access to parent values
        result = self.list_params(cmd_env, proj_name6, show_values=False)
        self.assertNotIn(param1, result.out())

        # cleanup -- need to unwind in order
        self.delete_project(cmd_env, proj_name6)
        self.delete_project(cmd_env, proj_name5)
        self.delete_project(cmd_env, proj_name4)
        self.delete_project(cmd_env, proj_name3)
        self.delete_project(cmd_env, proj_name2)
        self.delete_project(cmd_env, proj_name1)

    def test_project_pagination(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        page_size = TEST_PAGE_SIZE
        proj_count = page_size + 1

        for idx in range(proj_count):
            proj_name = self.make_name(f"test-pag-{idx}")
            self.create_project(cmd_env, proj_name)

        self.assertPaginated(cmd_env, base_cmd + "proj ls", "/projects/?")

        # cleanup
        for idx in range(proj_count):
            proj_name = self.make_name(f"test-pag-{idx}")
            self.delete_project(cmd_env, proj_name)

from testcase import TestCase


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
        self.assertIn(f"{proj_name},{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{proj_name},{new_desc}", result.out())

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

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(proj_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertResultWarning(result, f"Project '{proj_name}' does not exist")

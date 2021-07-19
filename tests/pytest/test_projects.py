from testcase import TestCase


class TestProjects(TestCase):
    def test_project_basic(self):
        # verify `proj_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-proj-name")
        sub_cmd = base_cmd + "projects "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertFalse(result.out_contains_value(proj_name))

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{orig_desc}\"")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_both(proj_name, orig_desc))

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{new_desc}\"")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_both(proj_name, new_desc))

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name} --desc \"{new_desc}\"")
        self.assertEqual(result.return_value, 0)

        # rename
        orig_name = proj_name
        proj_name = self.make_name("test-proj-rename")
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{proj_name}\"")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Updated project '{proj_name}'", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(
            f"Project '{proj_name}' not updated: no updated parameters provided",
            result.err(),
        )

        # test the list without the table
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(proj_name))
        self.assertFalse(result.out_contains_both(proj_name, new_desc))

        # test the csv output
        result = self.run_cli(cmd_env, sub_cmd + "list -v -f csv")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(proj_name))
        self.assertTrue(result.out_contains_both(proj_name, new_desc))

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertFalse(result.out_contains_value(proj_name))

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.err_contains_value(f"Project '{proj_name}' does not exist"))

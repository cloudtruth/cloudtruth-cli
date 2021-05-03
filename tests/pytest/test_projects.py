from testcase import TestCase
from testcase import DEFAULT_PROJ_NAME, DEFAULT_ENV_NAME


class TestProjects(TestCase):
    def test_project_basic(self):
        # verify `proj_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = "test-proj-name" #  TODO: make unique by datetime? 
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
        self.assertTrue(result.err_contains_value(f"Project '{proj_name}' not updated: same description"))

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {proj_name}")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.err_contains_value(f"Project '{proj_name}' not updated: no description"))

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

        # delete the description
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertFalse(result.out_contains_value(proj_name))
    
        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {proj_name} --confirm")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.err_contains_value(f"Project '{proj_name}' does not exist"))

    def test_cannot_delete_default(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        # set the proj/env to 'default', and do not expose secrets
        param_cmd = base_cmd + f"--project {DEFAULT_PROJ_NAME} --env {DEFAULT_ENV_NAME} param ls -v"

        # get an original snapshot (do not expose secrets)
        before = self.run_cli(cmd_env, param_cmd)

        # attempt to delete the default project and see failure
        result = self.run_cli(cmd_env, base_cmd + f"project delete {DEFAULT_PROJ_NAME} --confirm")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Cannot delete the default project", result.err())

        # make sure we get the same parameter list
        after = self.run_cli(cmd_env, param_cmd)
        self.assertEqual(before, after)

from testcase import TestCase
from testcase import DEFAULT_PROJ_NAME, DEFAULT_ENV_NAME


class TestEnvironments(TestCase):
    def test_environment_basic(self):
        # verify `env_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        env_name = self.make_name("test-env-name")
        sub_cmd = base_cmd + "environments "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertFalse(result.out_contains_value(env_name))

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name} --desc \"{orig_desc}\"")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_both(env_name, orig_desc))

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name} --desc \"{new_desc}\"")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_both(env_name, new_desc))

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name} --desc \"{new_desc}\"")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.err_contains_value(f"Environment '{env_name}' not updated: same description"))

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name}")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.err_contains_value(f"Environment '{env_name}' not updated: no description"))

        # test the list without the table
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(env_name))
        self.assertFalse(result.out_contains_both(env_name, new_desc))

        # test the csv output
        result = self.run_cli(cmd_env, sub_cmd + "list -v -f csv")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_both(env_name, new_desc))

        # delete the description
        result = self.run_cli(cmd_env, sub_cmd + f"delete {env_name} --confirm")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertFalse(result.out_contains_value(env_name))
    
        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {env_name} --confirm")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.err_contains_value(f"Environment '{env_name}' does not exist"))

    def test_cannot_delete_default(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        # set the proj/env to 'default', and do not expose secrets
        param_cmd = base_cmd + f"--project {DEFAULT_PROJ_NAME} --env {DEFAULT_ENV_NAME} param ls -v"

        # get an original snapshot (do not expose secrets)
        before = self.run_cli(cmd_env, param_cmd)

        # attempt to delete the default project and see failure
        result = self.run_cli(cmd_env, base_cmd + f"environment delete {DEFAULT_ENV_NAME} --confirm")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Cannot delete the default environment", result.err())

        # make sure we get the same parameter list
        after = self.run_cli(cmd_env, param_cmd)
        self.assertEqual(before, after)

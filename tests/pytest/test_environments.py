import unittest

from testcase import TestCase
from testcase import DEFAULT_ENV_NAME


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

        # rename the environment
        orig_name = env_name
        env_name = self.make_name("test-env-rename")
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{env_name}\"")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Updated environment '{env_name}'", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(
            f"Environment '{env_name}' not updated: no updated parameters provided",
            result.err(),
        )

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

    def test_environment_cannot_delete_default(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("default-env-del-test")
        self.create_project(cmd_env, proj_name)

        # set the proj/env to 'default', and do not expose secrets
        param_cmd = base_cmd + f"--project '{proj_name}' --env '{DEFAULT_ENV_NAME}' param ls -v"

        # get an original snapshot (do not expose secrets)
        before = self.run_cli(cmd_env, param_cmd)

        # attempt to delete the default project and see failure
        result = self.run_cli(cmd_env, base_cmd + f"environment delete '{DEFAULT_ENV_NAME}' --confirm")
        self.assertNotEqual(result.return_value, 0)
        # TODO: fix error message
        # self.assertIn("Cannot delete the default environment", result.err())

        # make sure we get the same parameter list
        after = self.run_cli(cmd_env, param_cmd)
        self.assertEqual(before, after)

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_environment_parents(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        env_name1 = self.make_name("cloud")
        env_name2 = self.make_name("truth")
        env_name3 = self.make_name("cli")
        env_name4 = self.make_name("gui")

        self.create_environment(cmd_env, env_name1)
        self.create_environment(cmd_env, env_name2, parent=env_name1)
        self.create_environment(cmd_env, env_name3, parent=env_name2)
        self.create_environment(cmd_env, env_name4, parent=env_name2)

        # Use csv to validate, since the names may be variable
        result = self.run_cli(cmd_env, base_cmd + "env ls -v -f csv")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{env_name1},{DEFAULT_ENV_NAME},", result.out())
        self.assertIn(f"{env_name2},{env_name1},", result.out())
        self.assertIn(f"{env_name3},{env_name2},", result.out())
        self.assertIn(f"{env_name4},{env_name2},", result.out())

        # attempt to delete something that is used elsewhere
        result = self.run_cli(cmd_env, base_cmd + f"environment delete '{env_name2}' --confirm")
        self.assertNotEqual(result.return_value, 0)
        # TODO: fix error message
        # self.assertIn("base Environments with children cannot be deleted", result.err())

        # attempt to create without an existing parent
        env_name5 = self.make_name("general")
        env_name6 = self.make_name("truthiness")
        result = self.run_cli(cmd_env, base_cmd + f"environments set '{env_name5}' --parent '{env_name6}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"No parent environment '{env_name6}' found", result.err())

        # attempt to update parent -- not allowed
        result = self.run_cli(cmd_env, base_cmd + f"environment set '{env_name4}' --parent '{env_name1}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"Environment '{env_name4}' parent cannot be updated", result.err())

        # setting to same parent is ignored
        new_desc = "My new description"
        result = self.run_cli(cmd_env, base_cmd + f"environment set '{env_name4}' --parent '{env_name2}' --desc '{new_desc}'")
        self.assertEqual(result.return_value, 0)

        # make sure description was updated, yet parent remains
        result = self.run_cli(cmd_env, base_cmd + "env ls -v -f csv")
        self.assertIn(f"{env_name4},{env_name2},{new_desc}", result.out())

        # cleanup -- need to unwind in order
        self.delete_environment(cmd_env, env_name4)
        self.delete_environment(cmd_env, env_name3)
        self.delete_environment(cmd_env, env_name2)
        self.delete_environment(cmd_env, env_name1)


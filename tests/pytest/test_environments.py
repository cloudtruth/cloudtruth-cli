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
        self.assertResultSuccess(result)
        self.assertNotIn(env_name, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name} --desc \"{orig_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name},default,{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name},default,{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        # rename the environment
        orig_name = env_name
        env_name = self.make_name("test-env-rename")
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{env_name}\"")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated environment '{env_name}'", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {env_name}")
        self.assertResultWarning(
            result,
            f"Environment '{env_name}' not updated: no updated parameters provided",
        )

        # test the list without the values
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(env_name, result.out())
        self.assertNotIn(new_desc, result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At", result.out())
        self.assertIn(env_name, result.out())
        self.assertIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {env_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(env_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {env_name} --confirm")
        self.assertResultWarning(result, f"Environment '{env_name}' does not exist")

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
        self.assertResultError(result, "Cannot delete the default environment")

        # make sure we get the same parameter list
        after = self.run_cli(cmd_env, param_cmd)
        self.assertEqual(before.return_value, after.return_value)
        self.assertEqual(before.out(), after.out())
        self.assertEqual(before.err(), after.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_environment_parents(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        env_name1 = self.make_name("cloud")
        env_name2 = self.make_name("truth")
        env_name3 = self.make_name("gui")
        env_name4 = self.make_name("cli")

        self.create_environment(cmd_env, env_name1)
        self.create_environment(cmd_env, env_name2, parent=env_name1)
        self.create_environment(cmd_env, env_name3, parent=env_name2)
        self.create_environment(cmd_env, env_name4, parent=env_name2)

        # Use csv to validate, since the names may be variable
        result = self.run_cli(cmd_env, base_cmd + "env ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name1},{DEFAULT_ENV_NAME},", result.out())
        self.assertIn(f"{env_name2},{env_name1},", result.out())
        self.assertIn(f"{env_name3},{env_name2},", result.out())
        self.assertIn(f"{env_name4},{env_name2},", result.out())

        # basic 'tree' test
        result = self.run_cli(cmd_env, base_cmd + "env tree")
        self.assertResultSuccess(result)
        expected = f"  {env_name1}\n    {env_name2}\n      {env_name4}\n      {env_name3}\n"
        self.assertIn(expected, result.out())

        # specifying the environment gets a filtered set
        result = self.run_cli(cmd_env, base_cmd + f"env tree '{env_name2}'")
        self.assertResultSuccess(result)
        expected = f"{env_name2}\n  {env_name4}\n  {env_name3}\n"
        self.assertEqual(expected, result.out())

        # invalid environment given
        result = self.run_cli(cmd_env, base_cmd + "env tree non-env")
        self.assertResultWarning(result, "No environment 'non-env' found")

        # attempt to delete something that is used elsewhere
        result = self.run_cli(cmd_env, base_cmd + f"environment delete '{env_name2}' --confirm")
        self.assertResultError(result, "Cannot remove environment because it has children")

        # attempt to create without an existing parent
        env_name5 = self.make_name("general")
        env_name6 = self.make_name("truthiness")
        result = self.run_cli(cmd_env, base_cmd + f"environments set '{env_name5}' --parent '{env_name6}'")
        self.assertResultError(result, f"No parent environment '{env_name6}' found")

        # attempt to update parent -- not allowed
        result = self.run_cli(cmd_env, base_cmd + f"environment set '{env_name4}' --parent '{env_name1}'")
        self.assertResultError(result, f"Environment '{env_name4}' parent cannot be updated")

        # setting to same parent is ignored
        new_desc = "My new description"
        cmd = base_cmd + f"environment set '{env_name4}' --parent '{env_name2}' --desc '{new_desc}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        # make sure description was updated, yet parent remains
        result = self.run_cli(cmd_env, base_cmd + "env ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name4},{env_name2},{new_desc}", result.out())

        # cleanup -- need to unwind in order
        self.delete_environment(cmd_env, env_name4)
        self.delete_environment(cmd_env, env_name3)
        self.delete_environment(cmd_env, env_name2)
        self.delete_environment(cmd_env, env_name1)

    def test_environment_tagging(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name = self.make_name("proj-env-tag")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("test-env-tag")
        self.create_environment(cmd_env, env_name)

        # set a parameter
        param_cmd = base_cmd + f"--env {env_name} --project '{proj_name}' param "
        param1 = "my-param"
        value1 = "temp value"
        result = self.run_cli(cmd_env, param_cmd + f"set '{param1}' -v '{value1}'")
        self.assertResultSuccess(result)

        # make sure it is empty
        tag_cmd = base_cmd + "env tag "
        list_cmd = tag_cmd + f"list '{env_name}' "
        empty_msg = f"No tags found in environment {env_name}"
        for cmd in [list_cmd, list_cmd + "-v", list_cmd + "-f csv", list_cmd + "-u", list_cmd + "-vuf csv"]:
            result = self.run_cli(cmd_env, cmd)
            self.assertResultSuccess(result)
            self.assertIn(empty_msg, result.out())

        # create tag
        tag1 = "my-orig-tag"
        desc1a = "first description"
        result = self.run_cli(cmd_env, tag_cmd + f"set '{env_name}' '{tag1}' -d '{desc1a}'")
        self.assertResultSuccess(result)

        # make sure we can see the entry
        result = self.run_cli(cmd_env, list_cmd + "-vf csv")
        self.assertResultSuccess(result)
        self.assertIn(tag1, result.out())
        self.assertIn(desc1a, result.out())
        orig_time = result.stdout[1].split(",")[1]

        # update the description
        desc1b = "make it stop"
        result = self.run_cli(cmd_env, tag_cmd + f"set '{env_name}' '{tag1}' -d '{desc1b}'")
        self.assertResultSuccess(result)

        # make sure we can see the entry, before it has been read
        usage_cmd = list_cmd + "-uf json"
        result = self.run_cli(cmd_env, usage_cmd)
        self.assertResultSuccess(result)
        tag_data = eval(result.out()).get("environment-tags")[0]
        self.assertEqual(tag1, tag_data.get("Name"))
        self.assertEqual(desc1b, tag_data.get("Description"))  # updated
        self.assertEqual(orig_time, tag_data.get("Timestamp"))  # not updated
        self.assertEqual("0", tag_data.get("Total Reads"))
        self.assertEqual("", tag_data.get("Last User"))
        # self.assertEqual("", tag_data.get("Last Time"))

        # just do a sample read using the tag
        param_list = param_cmd + f"ls --as-of '{tag1}' "
        result = self.run_cli(cmd_env, param_list + "-v")
        self.assertResultSuccess(result)
        self.assertIn(param1, result.out())
        self.assertIn(value1, result.out())

        # do it again without values (issue seen during development)
        result = self.run_cli(cmd_env, param_list)
        self.assertResultSuccess(result)
        self.assertIn(param1, result.out())
        self.assertNotIn(value1, result.out())

        # see the tag usage is updated
        result = self.run_cli(cmd_env, usage_cmd)
        self.assertResultSuccess(result)
        tag_data = eval(result.out()).get("environment-tags")[0]
        self.assertEqual("2", tag_data.get("Total Reads"))
        self.assertNotEqual("", tag_data.get("Last User"))
        self.assertNotEqual("", tag_data.get("Last Time"))

        # set a timestamp -- we are pretty liberal about what we convert
        timestamp = "03/24/2021"
        result = self.run_cli(cmd_env, tag_cmd + f"set {env_name} {tag1} -t '{timestamp}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd + "-u -f csv")
        self.assertResultSuccess(result)
        self.assertIn("2021-03-24", result.out())
        self.assertIn(",Total Reads,Last User,Last Time", result.out())

        # invalid timestamps -- make sure CLI catches the invalid things
        for timestamp in ["abcd", "2000"]:
            result = self.run_cli(cmd_env, tag_cmd + f"set {env_name} {tag1} -t '{timestamp}'")
            self.assertResultError(result, "Invalid time value")

        # delete the tag
        del_cmd = tag_cmd + f"del {env_name} {tag1} -y"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultSuccess(result)

        # idempotent
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultWarning(result, f"Environment '{env_name}' does not have a tag '{tag1}'")

        # set/list/delete unknown environment
        bad_env = "nonesuch"
        no_env_msg = f"The '{bad_env}' environment could not be found in your account"
        result = self.run_cli(cmd_env, tag_cmd + f"list {bad_env}")
        self.assertResultError(result, no_env_msg)
        result = self.run_cli(cmd_env, tag_cmd + f"set {bad_env} {tag1}")
        self.assertResultError(result, no_env_msg)
        result = self.run_cli(cmd_env, tag_cmd + f"delete {bad_env} {tag1}")
        self.assertResultWarning(result, f"Environment '{bad_env}' does not exist")

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

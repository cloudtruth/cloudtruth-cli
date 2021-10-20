from testcase import TestCase


class TestUsers(TestCase):
    def test_user_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        user_name = self.make_name("test-user-name")
        sub_cmd = base_cmd + "users "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(user_name, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --desc \"{orig_desc}\"")
        self.assertResultSuccess(result)
        api_key = result.stdout[1]

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{user_name},service,", result.out())
        self.assertIn(f"@cloudtruth.com,{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{user_name},service,", result.out())
        self.assertIn(f"@cloudtruth.com,{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        # use the new API key -- see we get some environments back
        result = self.run_cli(cmd_env, base_cmd + f"--api-key {api_key} env ls -vf csv")
        self.assertResultSuccess(result)

        # since the default is a 'viewer' role, see that we cannot set ourself to owner
        permission_err = "You do not have permission to perform this action"
        cmd = base_cmd + f"--api-key '{api_key}' user set '{user_name}' --role owner"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, permission_err)

        # update the role (though not visible)
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --role contrib")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated user '{user_name}'", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name}")
        self.assertResultWarning(
            result,
            f"User '{user_name}' not updated: no updated parameters provided",
        )

        # use the new API key -- see we get some environments back
        result = self.run_cli(cmd_env, base_cmd + f"--api-key {api_key} env ls -vf csv")
        self.assertResultSuccess(result)

        # try creating a new owner
        user2_name = self.make_name("test-another")
        cmd = base_cmd + f"--api-key '{api_key}' user set '{user2_name}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, permission_err)

        # test the list without the values
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(user_name, result.out())
        self.assertNotIn(user2_name, result.out())
        self.assertNotIn(new_desc, result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At,Last Used At", result.out())
        self.assertIn(user_name, result.out())
        self.assertIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {user_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(user_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {user_name} --confirm")
        self.assertResultWarning(result, f"User '{user_name}' does not exist")

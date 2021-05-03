from testcase import TestCase

CT_URL = "CLOUDTRUTH_SERVER_URL"

class TestRun(TestCase):

    def test_run_inheritance_env_only(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = "run-env-proj"
        param_name = "SOME_PARAM_NAME"
        env_value = "env_value"

        sub_cmd = base_cmd + f"--project {proj_name} run "
        print_env = " -c printenv"

        self.create_project(cmd_env, proj_name)

        # add the value to the run environment, and see it does not get in without inheritance
        cmd_env[param_name] = env_value
        result = self.run_cli(cmd_env, sub_cmd + "--inherit none" + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertNotIn(param_name, result.out())

        for inherit in ["underlay", "overlay", "exclusive"]:
            result = self.run_cli(cmd_env, sub_cmd + f"--inherit {inherit}" + print_env)
            self.assertEqual(result.return_value, 0)
            self.assertIn(f"{param_name}={env_value}", result.out())

        self.delete_project(cmd_env, proj_name)

    def test_run_inheritance_coordination(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = "run-inherit-proj"
        param_name = "SOME_PARAM_NAME"
        env_value = "env_value"
        ct_value="ct_value"
        env_str = f"{param_name}={env_value}"
        ct_str = f"{param_name}={ct_value}"

        sub_cmd = base_cmd + f"--project {proj_name} run "
        print_env = " -- printenv"

        cmd_env[param_name] = env_value  # add to the run environment
        self.create_project(cmd_env, proj_name)
        self.set_param(cmd_env, proj_name, param_name, ct_value)

        result = self.run_cli(cmd_env, sub_cmd + "--inherit none" + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertIn(ct_str, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--inherit underlay" + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertIn(env_str, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--inherit overlay" + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertIn(ct_str, result.out())

        # unspecified is the same as overlay
        result = self.run_cli(cmd_env, sub_cmd + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertIn(ct_str, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--inherit exclusive" + print_env)
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"Conflicting definitions in run environment for: {param_name}", result.err())

        self.delete_param(cmd_env, proj_name, param_name)
        self.delete_project(cmd_env, proj_name)

    def test_run_permissive(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = "run-permissive-proj"
        sub_cmd = base_cmd + f"--project {proj_name} run "
        print_env = " -- printenv"

        # make sure we have something that normally gets removed
        if CT_URL not in cmd_env:
            cmd_env[CT_URL] = "https://api.cloudtruth.com/graphql"
        self.create_project(cmd_env, proj_name)

        result = self.run_cli(cmd_env, sub_cmd + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertNotIn(CT_URL, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--permissive" + print_env)
        self.assertEqual(result.return_value, 0)
        self.assertIn(CT_URL, result.out())

        self.delete_project(cmd_env, proj_name)

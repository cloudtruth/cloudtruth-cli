from testcase import TestCase


class TestIntegrations(TestCase):
    def test_integration_explore_errors(self):
        base_cmd = self.get_cli_base_cmd()
        exp_cmd = base_cmd + "integrations explore "
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-int-explore-errors")
        self.create_project(cmd_env, proj_name)

        # check that we get notification about no provider
        fqn = 'test://missing.provider/should-gets-warning'
        result = self.run_cli(cmd_env, exp_cmd + f"-v '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"No integration provider for `{fqn}`", result.err())

        # check that we get notification about no provider
        fqn = 'github://missing.provider/should-gets-warning'
        result = self.run_cli(cmd_env, exp_cmd + f"-v '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"Integration for `{fqn}` could not be found", result.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)

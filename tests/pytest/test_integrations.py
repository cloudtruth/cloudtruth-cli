import unittest

from testcase import TestCase
from urllib.parse import urlparse


class TestIntegrations(TestCase):
    def setUp(self) -> None:
        self.fqn = "github://rickporter-tuono/cloudtruth_test/main/short.yaml"
        self.jmes = "speicla.POrk_Egg_Foo_Young"
        super().setUp()

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
        self.assertIn(f"No integration provider available for `{fqn}`", result.err())

        # check that we get notification about no provider
        fqn = 'github://missing.provider/should-gets-warning'
        result = self.run_cli(cmd_env, exp_cmd + f"-v '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"No integration available for `{fqn}`", result.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)

    @unittest.skip("Need known integration parameters")
    def test_integration_explore(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        url = urlparse(self.fqn)
        base_fqn = f"{url.scheme}://{url.netloc}"

        # make sure we see the integration in the list
        result = self.run_cli(cmd_env, base_cmd + "int ls")
        self.assertEqual(0, result.return_value)
        self.assertIn(f"{url.hostname}", result.out())

        # do it again with the CSV to see name and a baseline fqn
        result = self.run_cli(cmd_env, base_cmd + "integ ls -v --format csv")
        self.assertEqual(0, result.return_value)
        self.assertIn(f"{url.hostname},{base_fqn}/,", result.out())

        # now, walk the path
        explore_cmd = base_cmd + "int ex -v -f csv "
        path_parts = [_ for _ in url.path.replace("/", "", 1).split("/") if _]
        explore_path = base_fqn
        for name in path_parts:
            expected = f"{name},{explore_path}/{name}"
            result = self.run_cli(cmd_env, explore_cmd + f"'{explore_path}'")
            self.assertEqual(0, result.return_value)
            self.assertIn(expected, result.out())

            # update for next iteration
            explore_path += "/" + name

        # in the "final" pass, it should contain the JMES path
        expected = f"  {{{{ {self.jmes} }}}},{self.fqn}"
        result = self.run_cli(cmd_env, explore_cmd + f"'{explore_path}'")
        self.assertEqual(0, result.return_value)
        self.assertIn(expected, result.out())

    @unittest.skip("Need known integration parameters")
    def test_integration_parameters(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-param-names")
        empty_msg = f"No parameters found in project {proj_name}"
        param_cmd = base_cmd + f"--project '{proj_name}' param "
        show_cmd = param_cmd + "list -vsf csv"
        show_dyn = show_cmd + " --dynamic"

        # add a new project
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(empty_msg, result.out())

        ######################
        # start with a boring static value
        param1 = "pi"
        value1 = "3.14159"
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -v {value1}")
        self.assertEqual(0, result.return_value)

        result = self.run_cli(cmd_env, show_cmd)
        self.assertIn(f"{param1},{value1}", result.out())

        # see there are not dynamic parameters
        result = self.run_cli(cmd_env, show_dyn)
        self.assertIn("No dynamic parameters found in project", result.out())

        ######################
        # flip it to a dynamic value
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -f {self.fqn} -j {self.jmes}")
        self.assertEqual(0, result.return_value)
        self.assertIn("Successfully update", result.out())

        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(0, result.return_value)
        self.assertIn(f"{param1},", result.out())
        self.assertNotIn(value1, result.out())

        # see the dynamic parameter
        result = self.run_cli(cmd_env, show_dyn)
        expected = f"{param1},{self.fqn},{self.jmes}"
        self.assertIn(expected, result.out())

        ######################
        # flip back to static
        value2 = "are_round"
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -v {value2}")
        self.assertEqual(0, result.return_value)

        result = self.run_cli(cmd_env, show_cmd)
        self.assertIn(f"{param1},{value2}", result.out())

        # see there are not dynamic parameters
        result = self.run_cli(cmd_env, show_dyn)
        self.assertIn("No dynamic parameters found in project", result.out())

        ######################
        # create a dynamic value
        param2 = "eulers"
        result = self.run_cli(cmd_env, param_cmd + f"set {param2} -f {self.fqn} -j {self.jmes}")
        self.assertEqual(0, result.return_value)
        self.assertIn("Successfully update", result.out())

        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(0, result.return_value)
        self.assertIn(f"{param1},{value2}", result.out())
        self.assertIn(f"{param2},", result.out())
        self.assertNotIn(value1, result.out())

        # see the dynamic parameter
        result = self.run_cli(cmd_env, show_dyn)
        expected = f"{param2},{self.fqn},{self.jmes}"
        self.assertIn(expected, result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)

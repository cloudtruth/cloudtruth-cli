import os
import unittest
from typing import List

from testcase import TestCase
from testcase import write_file
from urllib.parse import urlparse

CT_BROKEN_PROJ = "CLOUDTRUTH_TEST_BROKEN_PROJECT"
CT_BROKEN_TEMP = "CLOUDTRUTH_TEST_BROKEN_TEMPLATE"
CT_BROKEN_PARAM1 = "CLOUDTRUTH_TEST_BROKEN_PARAM1"
CT_BROKEN_PARAM2 = "CLOUDTRUTH_TEST_BROKEN_PARAM2"
CT_BROKEN_PARAM3 = "CLOUDTRUTH_TEST_BROKEN_PARAM3"
CT_BROKEN_VALUE1 = "CLOUDTRUTH_TEST_BROKEN_VALUE1"
CT_BROKEN_FQN2 = "CLOUDTRUTH_TEST_BROKEN_FQN2"
CT_BROKEN_FQN3 = "CLOUDTRUTH_TEST_BROKEN_FQN3"
CT_BROKEN_RUN = [
    CT_BROKEN_PROJ,
    CT_BROKEN_TEMP,
    CT_BROKEN_PARAM1,
    CT_BROKEN_PARAM2,
    CT_BROKEN_PARAM3,
    CT_BROKEN_VALUE1,
    CT_BROKEN_FQN2,
    CT_BROKEN_FQN3,
]

CT_EXP_FQN = "CLOUDTRUTH_TEST_EXPLORE_FQN"
CT_EXP_JMES = "CLOUDTRUTH_TEST_EXPLORE_JMES"
CT_EXPLORE_RUN = [
    CT_EXP_FQN,
    CT_EXP_JMES,
]

CT_PARAM_FQN = "CLOUDTRUTH_TEST_PARAMETERS_FQN"
CT_PARAM_JMES = "CLOUDTRUTH_TEST_PARAMETERS_JMES"
CT_PARAM_RUN = [
    CT_PARAM_FQN,
    CT_PARAM_JMES,
]


def missing_any(env_var_names: List[str]) -> bool:
    return not all([os.environ.get(x) for x in env_var_names])


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
        self.assertIn(f"No integration provider available for `{fqn}`", result.err())

        # check that we get notification about no provider
        fqn = 'github://missing.provider/should-gets-warning'
        result = self.run_cli(cmd_env, exp_cmd + f"-v '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"No integration available for `{fqn}`", result.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)

    @unittest.skipIf(missing_any(CT_EXPLORE_RUN), "Need all CT_EXPLORE_RUN parameters")
    def test_integration_explore(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        fqn = os.environ.get(CT_EXP_FQN)
        jmes = os.environ.get(CT_EXP_JMES)
        url = urlparse(fqn)
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
        expected = f"  {{{{ {jmes} }}}},{fqn}"
        result = self.run_cli(cmd_env, explore_cmd + f"'{explore_path}'")
        self.assertEqual(0, result.return_value)
        self.assertIn(expected, result.out())

    @unittest.skipIf(missing_any(CT_PARAM_RUN), "Need all CT_PARAM_RUN parameters")
    def test_integration_parameters(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-int-params")
        empty_msg = f"No parameters found in project {proj_name}"
        param_cmd = base_cmd + f"--project '{proj_name}' param "
        show_cmd = param_cmd + "list -vsf csv"
        show_dyn = show_cmd + " --dynamic"

        fqn = os.environ.get(CT_PARAM_FQN)
        jmes = os.environ.get(CT_PARAM_JMES)

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
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -f {fqn} -j {jmes}")
        self.assertEqual(0, result.return_value)
        self.assertIn("Successfully update", result.out())

        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(0, result.return_value)
        self.assertIn(f"{param1},", result.out())
        self.assertNotIn(value1, result.out())

        # see the dynamic parameter
        result = self.run_cli(cmd_env, show_dyn)
        expected = f"{param1},{fqn},{jmes}"
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
        result = self.run_cli(cmd_env, param_cmd + f"set {param2} -f {fqn} -j {jmes}")
        self.assertEqual(0, result.return_value)
        self.assertIn("Successfully update", result.out())

        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(0, result.return_value)
        self.assertIn(f"{param1},{value2}", result.out())
        self.assertIn(f"{param2},", result.out())
        self.assertNotIn(value1, result.out())

        # see the dynamic parameter
        result = self.run_cli(cmd_env, show_dyn)
        expected = f"{param2},{fqn},{jmes}"
        self.assertIn(expected, result.out())

        # param get shows the dynamic parameters
        result = self.run_cli(cmd_env, param_cmd + f"get '{param2}' --details")
        self.assertIn(f"FQN: {fqn}", result.out())
        self.assertIn(f"JMES-path: {jmes}", result.out())

        ######################
        # templates with dynamic parameters
        temp_cmd = base_cmd + f"--project '{proj_name}' template "
        temp_name = "my-int-temp"
        filename = "template.txt"
        body = """\
# this is a comment that references an external parameter
PARAMETER_2 = PARAM2
"""
        write_file(filename, body.replace("PARAM2", f"{{{{{param2}}}}}"))
        result = self.run_cli(cmd_env, temp_cmd + f"preview '{filename}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(body.replace("PARAM2\n", ""), result.out())  # evaluated to an unknown value

        # create the template
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp_name}' --body '{filename}'")
        self.assertEqual(result.return_value, 0)

        # get the evaluated template
        result = self.run_cli(cmd_env, temp_cmd + f"get '{temp_name}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(body.replace("PARAM2\n", ""), result.out())  # evaluated to an unknown value

        # cleanup
        os.remove(filename)
        self.delete_project(cmd_env, proj_name)

    @unittest.skipIf(missing_any(CT_BROKEN_RUN), "Need all CT_BROKEN_RUN parameters")
    def test_integration_broken(self):
        # NOTE: this test is a bit different than the others because everything needs to exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name = os.environ.get(CT_BROKEN_PROJ)
        temp_name = os.environ.get(CT_BROKEN_TEMP)
        param1 = os.environ.get(CT_BROKEN_PARAM1)
        value1 = os.environ.get(CT_BROKEN_VALUE1)
        param2 = os.environ.get(CT_BROKEN_PARAM2)
        fqn2 = os.environ.get(CT_BROKEN_FQN2)
        param3 = os.environ.get(CT_BROKEN_PARAM3)
        fqn3 = os.environ.get(CT_BROKEN_FQN3)

        # make sure everything exists in the "correct" state
        proj_cmd = base_cmd + f"--project {proj_name} "
        result = self.run_cli(cmd_env, proj_cmd + "projects ls")
        self.assertIn(proj_name, result.out())
        result = self.run_cli(cmd_env, proj_cmd + "templates ls")
        self.assertIn(temp_name, result.out())

        missing_fqn2 = f"The dynamic content of `{fqn2}` is not present"
        missing_param2 = f"{param2}: {missing_fqn2}"

        ##########################
        # parameter checks
        result = self.run_cli(cmd_env, proj_cmd + "param list -f csv")
        self.assertEqual(result.return_value, 0)
        self.assertIn(param1, result.out())
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())
        self.assertEqual(result.err(), "")  # no errors reported, since not getting values

        # parameter list should yield warnings, but still show everything
        result = self.run_cli(cmd_env, proj_cmd + "param list -vsf csv")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{param1},{value1},", result.out())
        self.assertIn(f"{param2},,", result.out())  # empty value reported
        self.assertIn(f"{param3},", result.out())  # do not worry about returned value
        self.assertIn(missing_param2, result.err())
        self.assertNotIn(param3, result.err())
        self.assertNotIn(fqn3, result.err())

        # list dynamic parameters with no values
        result = self.run_cli(cmd_env, proj_cmd + "param list --dynamic -f csv")
        self.assertEqual(result.return_value, 0)
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())
        self.assertEqual("", result.err())  # no warnings if not getting values

        # list dynamic parameters with FQN/JMES
        result = self.run_cli(cmd_env, proj_cmd + "param list --dynamic -vf csv")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{param2},{fqn2}", result.out())
        self.assertIn(f"{param3},{fqn3}", result.out())
        self.assertIn(missing_param2, result.err())

        # getting the broken parameter yields an empty value, and a warning
        result = self.run_cli(cmd_env, proj_cmd + f"param get '{param2}'")
        self.assertEqual(result.return_value, 0)
        self.assertEqual("\n", result.out())
        self.assertIn(missing_fqn2, result.err())

        ##########################
        # template checks
        filename = "preview.txt"

        result = self.run_cli(cmd_env, proj_cmd + f"template get '{temp_name}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(missing_param2, result.err())

        # copy current body into a file
        result = self.run_cli(cmd_env, proj_cmd + f"template get '{temp_name}' --raw")
        write_file(filename, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"template preview '{filename}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(missing_param2, result.err())

        # NOTE: do NOT delete the project!!!
        os.remove(filename)

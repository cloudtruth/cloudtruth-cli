import os
import unittest

from testcase import TestCase
from testcase import PROP_NAME
from testcase import PROP_TYPE
from testcase import PROP_VALUE
from testcase import find_by_prop
from testcase import missing_any
from urllib.parse import urlparse

CT_BROKEN_PROJ = "CLOUDTRUTH_TEST_BROKEN_PROJECT"
CT_BROKEN_TEMP = "CLOUDTRUTH_TEST_BROKEN_TEMPLATE"
CT_BROKEN_PARAM1 = "CLOUDTRUTH_TEST_BROKEN_PARAM1"
CT_BROKEN_PARAM2 = "CLOUDTRUTH_TEST_BROKEN_PARAM2"
CT_BROKEN_PARAM3 = "CLOUDTRUTH_TEST_BROKEN_PARAM3"
CT_BROKEN_PARAM4 = "CLOUDTRUTH_TEST_BROKEN_PARAM4"
CT_BROKEN_VALUE1 = "CLOUDTRUTH_TEST_BROKEN_VALUE1"
CT_BROKEN_VALUE2 = "CLOUDTRUTH_TEST_BROKEN_VALUE2"
CT_BROKEN_VALUE3 = "CLOUDTRUTH_TEST_BROKEN_VALUE3"
CT_BROKEN_FQN2 = "CLOUDTRUTH_TEST_BROKEN_FQN2"
CT_BROKEN_FQN3 = "CLOUDTRUTH_TEST_BROKEN_FQN3"
CT_BROKEN_JMES2 = "CLOUDTRUTH_TEST_BROKEN_JMES2"
CT_BROKEN_JMES3 = "CLOUDTRUTH_TEST_BROKEN_JMES3"
CT_BROKEN_RUN = [
    CT_BROKEN_PROJ,
    CT_BROKEN_TEMP,
    CT_BROKEN_PARAM1,
    CT_BROKEN_PARAM2,
    CT_BROKEN_PARAM3,
    CT_BROKEN_PARAM4,
    CT_BROKEN_VALUE1,
    CT_BROKEN_VALUE2,
    CT_BROKEN_VALUE3,
    CT_BROKEN_FQN2,
    CT_BROKEN_FQN3,
    CT_BROKEN_JMES2,
    CT_BROKEN_JMES3,
]

CT_EXP_FQN = "CLOUDTRUTH_TEST_EXPLORE_FQN"
CT_EXP_JMES = "CLOUDTRUTH_TEST_EXPLORE_JMES"
CT_EXP_VALUE = "CLOUDTRUTH_TEST_EXPLORE_VALUE"
CT_EXPLORE_RUN = [
    CT_EXP_FQN,
    CT_EXP_JMES,
    CT_EXP_VALUE,
]

CT_PARAM_FQN = "CLOUDTRUTH_TEST_PARAMETERS_FQN"
CT_PARAM_JMES = "CLOUDTRUTH_TEST_PARAMETERS_JMES"
CT_PARAM_RUN = [
    CT_PARAM_FQN,
    CT_PARAM_JMES,
]

CT_TEMP_FQN = "CLOUDTRUTH_TEST_TEMPLATE_FQN"
CT_TEMP_PARAM1 = "CLOUDTRUTH_TEST_TEMPLATE_PARAM1"
CT_TEMP_RUN = [
    CT_TEMP_FQN,
    CT_TEMP_PARAM1,
]

CT_BASIC_INTEG_NAME = "CLOUDTRUTH_TEST_BASIC_INTEGRATION_NAME"
CT_BASIC_BAD_INT_NAME = "CLOUDTRUTH_TEST_BASIC_BAD_INTEGRATION_NAME"
CT_BASIC_RUN = [
    CT_BASIC_INTEG_NAME,
    CT_BASIC_BAD_INT_NAME,
]


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
        self.assertResultError(result, f"No integration provider available for `{fqn}`")

        # check that we get notification about no provider
        fqn = 'github://missing.provider/should-gets-warning'
        result = self.run_cli(cmd_env, exp_cmd + f"-v '{fqn}'")
        self.assertResultError(result, f"No integration available for `{fqn}`")

        # cleanup
        self.delete_project(cmd_env, proj_name)

    @unittest.skipIf(missing_any(CT_EXPLORE_RUN), "Need all CT_EXPLORE_RUN parameters")
    def test_integration_explore_success(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        fqn = os.environ.get(CT_EXP_FQN)
        jmes = os.environ.get(CT_EXP_JMES)
        value = os.environ.get(CT_EXP_VALUE)
        url = urlparse(fqn)
        base_fqn = f"{url.scheme}://{url.netloc}"

        # make sure we see the integration in the list
        result = self.run_cli(cmd_env, base_cmd + "int ls")
        self.assertResultSuccess(result)
        self.assertIn(f"{url.hostname}", result.out())

        # do it again with the CSV to see name and a baseline fqn
        result = self.run_cli(cmd_env, base_cmd + "integ ls -v --format csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{url.hostname},{base_fqn}/,", result.out())

        # check that times are added
        result = self.run_cli(cmd_env, base_cmd + "integ ls -v --format csv --show-times")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At", result.out())
        self.assertIn(f"{url.hostname},{base_fqn}/,", result.out())

        # now, walk the path
        explore_cmd = base_cmd + "int ex -v -f csv "
        path_parts = [_ for _ in url.path.replace("/", "", 1).split("/") if _]
        explore_path = base_fqn
        for name in path_parts:
            expected = f"{name},{explore_path}/{name}"
            result = self.run_cli(cmd_env, explore_cmd + f"'{explore_path}'")
            self.assertResultSuccess(result)
            self.assertIn(expected, result.out())

            # update for next iteration
            explore_path += "/" + name

        # in the "final" pass, it should contain the JMES path
        expected = f"  {{{{ {jmes} }}}},{fqn}"
        result = self.run_cli(cmd_env, explore_cmd + f"'{explore_path}'")
        self.assertResultSuccess(result)
        self.assertIn(expected, result.out())

        # verify that we get a warning when trying to display --raw for a file
        result = self.run_cli(cmd_env, explore_cmd + f"'{base_fqn}' -r")
        self.assertResultWarning(result, "Raw content only works for a single file")

        # cannot verify output, but the --raw option should be successful (and nothing in stderr)
        result = self.run_cli(cmd_env, explore_cmd + f"'{explore_path}' --raw")
        self.assertResultSuccess(result)

        # one more time with JMES path, showing the value
        result = self.run_cli(cmd_env, explore_cmd + f"'{fqn}' -j '{jmes}' --raw")
        self.assertResultSuccess(result)
        self.assertIn(value, result.out())

    @unittest.skipIf(missing_any(CT_PARAM_RUN), "Need all CT_PARAM_RUN parameters")
    def test_integration_parameters(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-int-params")
        empty_msg = f"No parameters found in project {proj_name}"
        param_cmd = base_cmd + f"--project '{proj_name}' param "
        show_cmd = param_cmd + "list -vsf csv"
        show_ext = show_cmd + " --external"

        fqn = os.environ.get(CT_PARAM_FQN)
        jmes = os.environ.get(CT_PARAM_JMES)

        # add a new project
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        result = self.run_cli(cmd_env, show_cmd)
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        ######################
        # start with a boring internal value
        param1 = "pi"
        value1 = "3.14159"
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -v {value1}")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, show_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value1}", result.out())

        # see there are not external parameters
        result = self.run_cli(cmd_env, show_ext)
        self.assertResultSuccess(result)
        self.assertIn("No external parameters found in project", result.out())

        ######################
        # flip it to an external value
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -f {fqn} -j {jmes}")
        self.assertResultSuccess(result)
        self.assertIn("Updated parameter", result.out())

        result = self.run_cli(cmd_env, show_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},", result.out())
        self.assertNotIn(value1, result.out())

        # see the external parameter
        result = self.run_cli(cmd_env, show_ext)
        self.assertResultSuccess(result)
        expected = f"{param1},{fqn},{jmes}"
        self.assertIn(expected, result.out())

        ######################
        # flip back to internal
        value2 = "are_round"
        result = self.run_cli(cmd_env, param_cmd + f"set {param1} -v {value2}")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, show_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value2}", result.out())

        # see there are not external parameters
        result = self.run_cli(cmd_env, show_ext)
        self.assertResultSuccess(result)
        self.assertIn("No external parameters found in project", result.out())

        ######################
        # create a external value
        param2 = "eulers"
        result = self.run_cli(cmd_env, param_cmd + f"set {param2} -f {fqn} -j {jmes}")
        self.assertResultSuccess(result)
        self.assertIn("Set parameter", result.out())

        result = self.run_cli(cmd_env, show_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value2}", result.out())
        self.assertIn(f"{param2},", result.out())
        self.assertNotIn(value1, result.out())

        # see the external parameter
        result = self.run_cli(cmd_env, show_ext)
        self.assertResultSuccess(result)
        expected = f"{param2},{fqn},{jmes}"
        self.assertIn(expected, result.out())

        # param get shows the external parameter properties
        result = self.run_cli(cmd_env, param_cmd + f"get '{param2}' --details")
        self.assertResultSuccess(result)
        self.assertIn(f"FQN: {fqn}", result.out())
        self.assertIn(f"JMES-path: {jmes}", result.out())

        ######################
        # templates with external parameters
        temp_cmd = base_cmd + f"--project '{proj_name}' template "
        temp_name = "my-int-temp"
        filename = "template.txt"
        body = """\
# this is a comment that references an external parameter
PARAMETER_2 = PARAM2
"""
        self.write_file(filename, body.replace("PARAM2", f"{{{{{param2}}}}}"))
        result = self.run_cli(cmd_env, temp_cmd + f"preview '{filename}'")
        self.assertResultSuccess(result)
        self.assertIn(body.replace("PARAM2\n", ""), result.out())  # evaluated to an unknown value

        # create the template
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp_name}' --body '{filename}'")
        self.assertResultSuccess(result)

        # get the evaluated template
        result = self.run_cli(cmd_env, temp_cmd + f"get '{temp_name}'")
        self.assertResultSuccess(result)
        self.assertIn(body.replace("PARAM2\n", ""), result.out())  # evaluated to an unknown value

        # cleanup
        self.delete_file(filename)
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
        jmes2 = os.environ.get(CT_BROKEN_JMES2)
        value2 = os.environ.get(CT_BROKEN_VALUE2)
        param3 = os.environ.get(CT_BROKEN_PARAM3)
        fqn3 = os.environ.get(CT_BROKEN_FQN3)
        jmes3 = os.environ.get(CT_BROKEN_JMES3)
        value3 = os.environ.get(CT_BROKEN_VALUE3)
        param4 = self.make_name(os.environ.get(CT_BROKEN_PARAM4))

        # make sure everything exists in the "correct" state
        proj_cmd = base_cmd + f"--project {proj_name} "
        result = self.run_cli(cmd_env, proj_cmd + "projects ls")
        self.assertResultSuccess(result)
        self.assertIn(proj_name, result.out())
        result = self.run_cli(cmd_env, proj_cmd + "templates ls")
        self.assertResultSuccess(result)
        self.assertIn(temp_name, result.out())

        ##########################
        # verify the FQNs are not reachable
        result = self.run_cli(cmd_env, base_cmd + f"int exp '{fqn2}' -j '{jmes2}' -r")
        self.assertResultWarning(result, f"Nothing found for FQN '{fqn2}'")
        result = self.run_cli(cmd_env, base_cmd + f"int exp '{fqn3}' -j '{jmes3}' -r")
        self.assertResultWarning(result, f"Nothing found for FQN '{fqn3}'")

        ##########################
        # parameter checks
        entries = self.get_cli_entries(cmd_env, proj_cmd + "param list -vsf json", "parameter")
        entry = find_by_prop(entries, PROP_NAME, param1)[0]
        self.assertEqual(entry.get(PROP_VALUE), value1)
        entry = find_by_prop(entries, PROP_NAME, param2)[0]
        self.assertEqual(entry.get(PROP_VALUE), value2)
        entry = find_by_prop(entries, PROP_NAME, param3)[0]
        self.assertEqual(entry.get(PROP_VALUE), value3)

        # list external parameters with no values
        result = self.run_cli(cmd_env, proj_cmd + "param list --external")
        self.assertResultSuccess(result)
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())

        # list external parameters with FQN/JMES
        result = self.run_cli(cmd_env, proj_cmd + "param list --external -vf csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{param2},{fqn2},{jmes2}", result.out())
        self.assertIn(f"{param3},{fqn3},{jmes3}", result.out())

        # getting the broken parameter yields an empty value, and a warning
        result = self.run_cli(cmd_env, proj_cmd + f"param get '{param2}'")
        self.assertResultSuccess(result)
        self.assertIn(value2, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"param get '{param3}' -d")
        self.assertResultSuccess(result)
        self.assertIn(f"Value: {value3}", result.out())
        self.assertIn(f"FQN: {fqn3}", result.out())
        self.assertIn(f"JMES-path: {jmes3}", result.out())

        # export will fail, and should provide details about what failed
        result = self.run_cli(cmd_env, proj_cmd + "param export docker")
        self.assertResultSuccess(result)
        self.assertIn(f"{param1.upper()}={value1}", result.out())
        self.assertIn(f"{param2.upper()}={value2}", result.out())
        self.assertIn(f"{param3.upper()}={value3}", result.out())

        # see that adding param4 with a reference to param2 is allowed -- does not rely on evaluation
        value4 = f"{{{{{param2}}}}}"
        result = self.run_cli(cmd_env, proj_cmd + f"param set '{param4}' -v '{value4}' -e true")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, proj_cmd + "param list -vsf csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{param4},{value2}", result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"param del '{param4}' -y")
        self.assertResultSuccess(result, "Removed")

        # cannot assign with broken FQN
        result = self.run_cli(cmd_env, proj_cmd + f"param set '{param4}' -f '{fqn2}' -j '{jmes2}'")
        self.assertResultError(result, f"The external content of `{fqn2}` is not present")

        ##########################
        # template checks -- still works using "old" values

        # make sure the template contains references (not iron-clad, but worth something)
        result = self.run_cli(cmd_env, proj_cmd + f"template get '{temp_name}' --raw")
        self.assertResultSuccess(result)
        self.assertIn(param1, result.out())
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())

        # copy current body into a file
        filename = "preview.txt"
        self.write_file(filename, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"template get '{temp_name}'")
        self.assertResultSuccess(result)
        self.assertIn(value1, result.out())
        self.assertIn(value2, result.out())
        self.assertIn(value3, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"template validate '{temp_name}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, proj_cmd + f"template preview '{filename}'")
        self.assertResultSuccess(result)
        self.assertIn(value1, result.out())
        self.assertIn(value2, result.out())
        self.assertIn(value3, result.out())

        # NOTE: do NOT delete the project!!!
        self.delete_file(filename)

    @unittest.skipIf(missing_any(CT_TEMP_RUN), "Need all CT_TEMP_RUN parameters")
    def test_integration_external_template(self):
        # in this test, we create a parameter (param1) that has an external reference to a template
        # that references an internal parameter (param2)
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-int-ext-temp")
        self.create_project(cmd_env, proj_name)
        proj_cmd = base_cmd + f"--project '{proj_name}' "

        temp_fqn = os.environ.get(CT_TEMP_FQN)
        param1 = os.environ.get(CT_TEMP_PARAM1)
        value1 = "this-is the param1 value"
        param2 = "param-refs-ext-template"
        self.set_param(cmd_env, proj_name, param1, value1)
        self.set_param(cmd_env, proj_name, param2, evaluate=True, fqn=temp_fqn)

        # see the evaluated template shows up
        result = self.list_params(cmd_env, proj_name, fmt="json")
        entries = eval(result.out()).get("parameter")
        entry1 = [e for e in entries if e.get(PROP_NAME) == param1][0]
        self.assertEqual(entry1.get(PROP_VALUE), value1)
        self.assertEqual(entry1.get(PROP_TYPE), "internal")
        entry2 = [e for e in entries if e.get(PROP_NAME) == param2][0]
        self.assertIn(value1, entry2.get(PROP_VALUE))
        self.assertEqual(entry2.get(PROP_TYPE), "external-evaluated")

        result = self.run_cli(cmd_env, proj_cmd + f"param del -y '{param1}'")
        self.assertResultSuccess(result)  # TODO: should be error due to being referenced??

        # delete external parameter
        result = self.run_cli(cmd_env, proj_cmd + f"param del -y '{param2}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, proj_cmd + f"param del -y '{param1}'")
        self.assertResultSuccess(result)
        self.assertIn("Did not find parameter", result.out())  # TODO: message goes away

        # attempt adding in other order -- adding external template with broken references
        eval_err = "Evaluation error: Template contains references that do not exist"
        result = self.run_cli(cmd_env, proj_cmd + f"param set {param2} -f '{temp_fqn}' -e true")
        self.assertResultError(result, eval_err)
        self.assertIn(param1, result.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)

    @unittest.skipIf(missing_any(CT_BASIC_RUN), "Need all CT_BASIC_RUN parameters")
    def test_integration_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        integ_name = os.environ.get(CT_BASIC_INTEG_NAME)
        bad_int_name = os.environ.get(CT_BASIC_BAD_INT_NAME)

        entries = self.get_cli_entries(cmd_env, base_cmd + "integrations list -f json", "integration")
        entry = find_by_prop(entries, PROP_NAME, integ_name)
        self.assertIsNotNone(entry)

        ##########################
        # test integration get/refresh
        result = self.run_cli(cmd_env, base_cmd + f"integ get '{integ_name}'")
        self.assertResultSuccess(result)
        last_update = result.out_contains("Updated At:")
        last_status = result.out_contains("Value:")

        result = self.run_cli(cmd_env, base_cmd + f"int refresh '{integ_name}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Refreshed integration '{integ_name}'", result.out())

        result = self.run_cli(cmd_env, base_cmd + f"integ get '{integ_name}'")
        self.assertResultSuccess(result)
        next_update = result.out_contains("Updated At:")
        next_status = result.out_contains("Value:")

        # race condition with other tests, but the time does NOT get updated if it is already
        # in a "checking" state
        if "checking" not in last_status:
            self.assertNotEqual((last_status, last_update), (next_status, next_update))

        ##########################
        # check error cases with
        no_integration_msg = f"Integration '{bad_int_name}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"int get '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int refresh '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        # nothing to cleanup

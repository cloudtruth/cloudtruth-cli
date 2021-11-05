import os
import unittest
import subprocess

from typing import List

from testcase import TestCase
from testcase import PROP_CREATED
from testcase import PROP_NAME
from testcase import PROP_MODIFIED
from testcase import PROP_TYPE
from testcase import PROP_VALUE
from testcase import find_by_prop
from urllib.parse import urlparse

CT_BROKEN_PROJ = "CLOUDTRUTH_TEST_BROKEN_PROJECT"
CT_BROKEN_TEMP = "CLOUDTRUTH_TEST_BROKEN_TEMPLATE"
CT_BROKEN_PARAM1 = "CLOUDTRUTH_TEST_BROKEN_PARAM1"
CT_BROKEN_PARAM2 = "CLOUDTRUTH_TEST_BROKEN_PARAM2"
CT_BROKEN_PARAM3 = "CLOUDTRUTH_TEST_BROKEN_PARAM3"
CT_BROKEN_PARAM4 = "CLOUDTRUTH_TEST_BROKEN_PARAM4"
CT_BROKEN_VALUE1 = "CLOUDTRUTH_TEST_BROKEN_VALUE1"
CT_BROKEN_FQN2 = "CLOUDTRUTH_TEST_BROKEN_FQN2"
CT_BROKEN_FQN3 = "CLOUDTRUTH_TEST_BROKEN_FQN3"
CT_BROKEN_RUN = [
    CT_BROKEN_PROJ,
    CT_BROKEN_TEMP,
    CT_BROKEN_PARAM1,
    CT_BROKEN_PARAM2,
    CT_BROKEN_PARAM3,
    CT_BROKEN_PARAM4,
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

CT_TEMP_FQN = "CLOUDTRUTH_TEST_TEMPLATE_FQN"
CT_TEMP_PARAM1 = "CLOUDTRUTH_TEST_TEMPLATE_PARAM1"
CT_TEMP_RUN = [
    CT_TEMP_FQN,
    CT_TEMP_PARAM1,
]

CT_PUSH_INTEG_NAME = "CLOUDTRUTH_TEST_PUSH_INTEGRATION_NAME"
CT_PUSH_BAD_INT_NAME = "CLOUDTRUTH_TEST_PUSH_BAD_INTEGRATION_NAME"
CT_PUSH_RUN = [
    CT_PUSH_INTEG_NAME,
    CT_PUSH_BAD_INT_NAME,
]


def missing_any(env_var_names: List[str]) -> bool:
    return not all([os.environ.get(x) for x in env_var_names])


class TestIntegrations(TestCase):
    def __init__(self, *args, **kwargs):
        self._pushes = None
        super().__init__(*args, **kwargs)

    def setUp(self) -> None:
        self._pushes = list()
        super().setUp()

    def tearDown(self) -> None:
        # delete any possibly lingering pushes
        for entry in self._pushes:
            cmd = self._base_cmd + f"int push del \"{entry[0]}\" \"{entry[1]}\" -y"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        super().tearDown()

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
    def test_integration_explore(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        fqn = os.environ.get(CT_EXP_FQN)
        jmes = os.environ.get(CT_EXP_JMES)
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
        self.assertIn("Successfully updated parameter", result.out())

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
        self.assertIn("Successfully set parameter", result.out())

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
        param3 = os.environ.get(CT_BROKEN_PARAM3)
        fqn3 = os.environ.get(CT_BROKEN_FQN3)
        param4 = os.environ.get(CT_BROKEN_PARAM4)

        # make sure everything exists in the "correct" state
        proj_cmd = base_cmd + f"--project {proj_name} "
        result = self.run_cli(cmd_env, proj_cmd + "projects ls")
        self.assertResultSuccess(result)
        self.assertIn(proj_name, result.out())
        result = self.run_cli(cmd_env, proj_cmd + "templates ls")
        self.assertResultSuccess(result)
        self.assertIn(temp_name, result.out())

        missing_fqn2 = f"The external content of `{fqn2}` is not present"
        missing_param2 = f"{param2}: {missing_fqn2}"

        ##########################
        # parameter checks
        result = self.run_cli(cmd_env, proj_cmd + "param list")
        self.assertResultSuccess(result)
        self.assertIn(param1, result.out())
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())

        # parameter list should yield warnings, but still show everything
        result = self.run_cli(cmd_env, proj_cmd + "param list -sf csv")
        self.assertResultWarning(result, missing_param2)
        self.assertIn(f"{param1},{value1},", result.out())
        self.assertIn(f"{param2},,", result.out())  # empty value reported
        self.assertIn(f"{param3},", result.out())  # do not worry about returned value
        self.assertNotIn(param3, result.err())
        self.assertNotIn(fqn3, result.err())

        # list external parameters with no values
        result = self.run_cli(cmd_env, proj_cmd + "param list --external")
        self.assertResultSuccess(result)
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())

        # list external parameters with FQN/JMES
        result = self.run_cli(cmd_env, proj_cmd + "param list --external -vf csv")
        self.assertResultWarning(result, missing_param2)
        self.assertIn(f"{param2},{fqn2}", result.out())
        self.assertIn(f"{param3},{fqn3}", result.out())

        # getting the broken parameter yields an empty value, and a warning
        result = self.run_cli(cmd_env, proj_cmd + f"param get '{param2}'")
        self.assertResultWarning(result, missing_fqn2)
        self.assertEqual("\n", result.out())

        # export will fail, and should provide details about what failed
        result = self.run_cli(cmd_env, proj_cmd + "param export docker")
        self.assertResultError(result, missing_param2)

        # see that adding param4 with a reference to param2 is not allowed
        value4 = f"{{{{{param2}}}}}"
        result = self.run_cli(cmd_env, proj_cmd + f"param set '{param4}' -v '{value4}' -e true")
        self.assertResultError(result, missing_param2)

        result = self.run_cli(cmd_env, proj_cmd + "param list -vsf csv")
        self.assertResultWarning(result, missing_param2)
        self.assertNotIn(param4, result.out())

        ##########################
        # template checks
        filename = "preview.txt"

        result = self.run_cli(cmd_env, proj_cmd + f"template get '{temp_name}'")
        self.assertResultError(result, missing_param2)

        result = self.run_cli(cmd_env, proj_cmd + f"template validate '{temp_name}'")
        self.assertResultError(result, missing_param2)

        # copy current body into a file
        result = self.run_cli(cmd_env, proj_cmd + f"template get '{temp_name}' --raw")
        self.assertResultSuccess(result)
        # make sure the template has the references
        self.write_file(filename, result.out())
        self.assertIn(param1, result.out())
        self.assertIn(param2, result.out())
        self.assertIn(param3, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"template preview '{filename}'")
        self.assertResultError(result, missing_param2)

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
        eval_err = "Evaluation error: Template references parameter(s) that do not exist"
        result = self.run_cli(cmd_env, proj_cmd + f"param set {param2} -f '{temp_fqn}' -e true")
        self.assertResultError(result, eval_err)
        self.assertIn(param1, result.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)

    @unittest.skipIf(missing_any(CT_PUSH_RUN), "Need all CT_PUSH_RUN parameters")
    def test_integration_push_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        integ_name = os.environ.get(CT_PUSH_INTEG_NAME)
        bad_int_name = os.environ.get(CT_PUSH_BAD_INT_NAME)
        set_cmd = base_cmd + f"integration push set '{integ_name}' "

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

        self.assertNotEqual((last_status, last_update), (next_status, next_update))

        ########################
        # create a couple projects
        proj_name1 = self.make_name("test-push-proj1")
        self.create_project(cmd_env, proj_name1)
        proj_name2 = self.make_name("test-push-proj2")
        self.create_project(cmd_env, proj_name2)

        ########################
        # create a couple environments with tags
        env_name1 = self.make_name("push-env-left")
        self.create_environment(cmd_env, env_name1)
        env1_tag1 = "sna"
        self.create_env_tag(cmd_env, env_name1, env1_tag1)
        env1_tag2 = "foo"
        self.create_env_tag(cmd_env, env_name1, env1_tag2)

        env_name2 = self.make_name("push-env-right")
        self.create_environment(cmd_env, env_name2)
        env2_tag1 = "foo"
        self.create_env_tag(cmd_env, env_name2, env2_tag1)
        env2_tag2 = "bar"
        self.create_env_tag(cmd_env, env_name2, env2_tag2)

        ########################
        # create the push
        default_resource = "/{{ environment }}/{{ project }}/{{ parameter }}"
        default_service = "ssm"
        default_region = "us-east-1"
        push_name1 = self.make_name("my-test-push")
        desc1 = "original comment"
        resource1 = "/{{ environment }}/{{ project }}/{{ parameter }}"
        self._pushes.append((integ_name, push_name1))
        tag1 = f"{env_name1}:{env1_tag1}"
        cmd = set_cmd + f"{push_name1} -d '{desc1}' --resource '{resource1}' --project '{proj_name1}' --tag '{tag1}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created", result.out())

        ########################
        # check it was created
        list_cmd = base_cmd + f"integration push list '{integ_name}' "
        result = self.run_cli(cmd_env, list_cmd + "-f json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("integration-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name1)[0]
        self.assertEqual(entry.get("Projects"), proj_name1)
        self.assertEqual(entry.get("Tags"), f"{tag1}")

        # check the right values were set
        result = self.run_cli(cmd_env, base_cmd + f"int push get {integ_name} {push_name1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name1}", result.out())
        self.assertIn(f"Resource: {resource1}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Projects: {proj_name1}", result.out())
        self.assertIn(f"Tags: {tag1}", result.out())
        self.assertIn(f"Region: {default_region}", result.out())
        self.assertIn(f"Service: {default_service}", result.out())

        # rename push, change resource, add another project, and another tag
        push_name2 = self.make_name("updated-test-push")
        resource2 = "/{{ project }}/{{ parameter }}/{{ environment }}"
        self._pushes.append((integ_name, push_name2))
        tag2 = f"{env_name2}:{env2_tag1}"
        cmd = (
            set_cmd + f"'{push_name1}' --resource '{resource2}' -r '{push_name2}' "
            f"--project '{proj_name2}' --tag '{tag2}'"
        )
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Updated", result.out())

        # check we have one entry with updated values
        result = self.run_cli(cmd_env, list_cmd + "-f json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("integration-push")
        # original name does not exist
        self.assertEqual(0, len(find_by_prop(pushes, PROP_NAME, push_name1)))
        # check the updated entry
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertEqual(entry.get("Projects"), f"{proj_name1}, {proj_name2}")
        entry_tags = entry.get("Tags")
        self.assertIn(tag1, entry_tags)
        self.assertIn(tag2, entry_tags)
        self.assertEqual(len(entry_tags.split(' ')), 2)

        # check the right values were updated
        result = self.run_cli(cmd_env, base_cmd + f"int push get {integ_name} {push_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Projects: {proj_name1}, {proj_name2}", result.out())
        self.assertIn(f"Tags: {tag2}, {tag1}", result.out())

        # list without specifying the integration...
        result = self.run_cli(cmd_env, base_cmd + "int push ls --format json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("integration-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertEqual(entry.get("Integration"), integ_name)
        self.assertIsNone(entry.get(PROP_CREATED))
        self.assertIsNone(entry.get(PROP_MODIFIED))
        last_time = entry.get("Last Push Time")

        result = self.run_cli(cmd_env, base_cmd + f"int push sync '{integ_name}' '{push_name2}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Synchronized push '{push_name2}'", result.out())

        result = self.run_cli(cmd_env, base_cmd + f"int push ls '{integ_name}' --format json --show-times")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("integration-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertIsNone(entry.get("Integration"))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        self.assertNotEqual(entry.get("Last Push Time"), last_time)

        # change the description, remove a project, and play with tags
        desc2 = "Updated description"
        self._pushes.append((integ_name, push_name2))
        tag3 = f"{env_name1}:{env1_tag2}"
        # tag4 = f"{env_name2}:{env2_tag2}"  # TODO: fix resource checking vs tags
        cmd = (
            set_cmd + f"'{push_name2}' -d '{desc2}' --no-project '{proj_name1}' --no-tag '{tag1}' "
            f"--tag '{tag3}' "
            # f"--tag '{tag4}'"
        )
        '''
        confused_msg = "Multiple tags from the same environment in the same push action require using `{{ tag }}`"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, confused_msg)

        # so, change the resource string, too
        resource3 = f"{resource2}/{{{{ tag }}}}"
        cmd += f"--resource {resource3}"
        '''
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Updated", result.out())

        # check the right values were updated
        result = self.run_cli(cmd_env, base_cmd + f"int push get {integ_name} {push_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc2}", result.out())
        self.assertIn(f"Projects: {proj_name2}", result.out())

        # check the tags in more detail
        result = self.run_cli(cmd_env, list_cmd + "-f json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("integration-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        entry_tags = entry.get("Tags")
        self.assertNotIn(tag1, entry_tags)
        self.assertIn(tag2, entry_tags)
        self.assertIn(tag3, entry_tags)
        # self.assertIn(tag4, entry_tags)
        # self.assertEqual(len(entry_tags.split(' ')), 3)
        self.assertEqual(len(entry_tags.split(' ')), 2)

        ########################
        # task list
        result = self.run_cli(cmd_env, base_cmd + f"int push tasks '{integ_name}' '{push_name2}' -f json")
        self.assertResultSuccess(result)
        tasks = eval(result.out()).get("integration-push-task")
        self.assertGreaterEqual(len(tasks), 1)
        self.assertEqual(1, len(find_by_prop(tasks, "Reason", "push created")))
        entry = tasks[0]
        self.assertIsNotNone(entry.get("Reason"))
        self.assertIsNotNone(entry.get("State"))
        self.assertIsNotNone(entry.get("Status Info"))

        ########################
        # delete the push
        del_cmd = base_cmd + f"int push del '{integ_name}' '{push_name2}' -y"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultSuccess(result)
        self.assertIn("Deleted", result.out())

        # idempotent
        no_push_msg = f"Integration push '{push_name2}' not found in integration '{integ_name}'"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultWarning(result, no_push_msg)

        # make sure it is gone
        result = self.run_cli(cmd_env, list_cmd + "-f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{push_name1},", result.out())
        self.assertNotIn(f"{push_name2},", result.out())

        ########################
        # create another push -- different values:
        #       check default resource,
        #       secretsmanager service (non-default),
        #       different region (non-default)
        service = "secretsmanager"
        region = "us-west-2"
        cmd = (
            base_cmd + f"int push set '{integ_name}' '{push_name1}' --service '{service}' "
            f"--region {region}"
        )
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created push", result.out())

        # do a get to verify the values
        result = self.run_cli(cmd_env, base_cmd + f"int push get '{integ_name}' '{push_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Region: {region}", result.out())
        self.assertIn(f"Service: {service}", result.out())
        self.assertIn(f"Resource: {default_resource}", result.out())

        # delete this push
        result = self.run_cli(cmd_env, base_cmd + f"int push del '{integ_name}' '{push_name1}' -y")
        self.assertResultSuccess(result)

        ########################
        # invalid region
        bad_reg_cmd = base_cmd + f"int push set '{integ_name}' '{push_name2}' --region not-a-region"
        result = self.run_cli(cmd_env, bad_reg_cmd)
        self.assertResultError(result, "isn't a valid value for '--region <region>'")

        ########################
        # no project found
        bogus_project = "this-proj-dne"
        bad_proj_cmd = base_cmd + f"int push set '{integ_name}' '{push_name2}' --project {bogus_project}"
        result = self.run_cli(cmd_env, bad_proj_cmd)
        self.assertResultError(result, f"Project '{bogus_project}' not found")

        ########################
        # invalid tag formats
        pre_tag_cmd = base_cmd + f"int push set '{integ_name}' '{push_name2}' "
        result = self.run_cli(cmd_env, pre_tag_cmd + "--tag foo")
        self.assertResultError(result, "Use a ':' to separate the environment and tag names")

        result = self.run_cli(cmd_env, pre_tag_cmd + "--tag sna:foo:bar")
        self.assertResultError(result, "Can only have one ':' to separate the environment and tag names")

        ########################
        # error out for invalid push name
        result = self.run_cli(cmd_env, base_cmd + f"int push task '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int push get '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int push sync '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        ########################
        # error out for bad integration name
        no_integration_msg = f"Integration '{bad_int_name}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"int p l '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int get '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int refresh '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int p get '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int p set '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int p sync '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int p task '{bad_int_name}' '{push_name1}' -v")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"int p del '{bad_int_name}' '{push_name1}' -y")
        self.assertResultError(result, no_integration_msg)

        # cleanup
        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)
        self.delete_environment(cmd_env, env_name1)
        self.delete_environment(cmd_env, env_name2)

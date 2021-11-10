import os
import unittest
import subprocess

from typing import List

from testcase import TestCase
from testcase import PROP_CREATED
from testcase import PROP_NAME
from testcase import PROP_MODIFIED
from testcase import find_by_prop

CT_PUSH_INTEG_NAME = "CLOUDTRUTH_TEST_PUSH_INTEGRATION_NAME"
CT_PUSH_BAD_INT_NAME = "CLOUDTRUTH_TEST_PUSH_BAD_INTEGRATION_NAME"
CT_PUSH_RUN = [
    CT_PUSH_INTEG_NAME,
    CT_PUSH_BAD_INT_NAME,
]


def missing_any(env_var_names: List[str]) -> bool:
    return not all([os.environ.get(x) for x in env_var_names])


class TestActions(TestCase):
    def __init__(self, *args, **kwargs):
        self._pushes = None
        super().__init__(*args, **kwargs)

    def setUp(self) -> None:
        self._pushes = list()
        super().setUp()

    def tearDown(self) -> None:
        # delete any possibly lingering pushes
        for entry in self._pushes:
            cmd = self._base_cmd + f"act push del \"{entry[0]}\" \"{entry[1]}\" -y"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        super().tearDown()

    @unittest.skipIf(missing_any(CT_PUSH_RUN), "Need all CT_PUSH_RUN parameters")
    def test_action_push_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        integ_name = os.environ.get(CT_PUSH_INTEG_NAME)
        bad_int_name = os.environ.get(CT_PUSH_BAD_INT_NAME)
        set_cmd = base_cmd + f"actions push set '{integ_name}' "

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
        list_cmd = base_cmd + f"action push list '{integ_name}' "
        result = self.run_cli(cmd_env, list_cmd + "-f json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("action-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name1)[0]
        self.assertEqual(entry.get("Projects"), proj_name1)
        self.assertEqual(entry.get("Tags"), f"{tag1}")

        # check the right values were set
        result = self.run_cli(cmd_env, base_cmd + f"act push get {integ_name} {push_name1}")
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
        pushes = eval(result.out()).get("action-push")
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
        result = self.run_cli(cmd_env, base_cmd + f"ac push get {integ_name} {push_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Projects: {proj_name1}, {proj_name2}", result.out())
        self.assertIn(f"Tags: {tag2}, {tag1}", result.out())

        # list without specifying the integration...
        result = self.run_cli(cmd_env, base_cmd + "act push ls --format json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("action-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertEqual(entry.get("Integration"), integ_name)
        self.assertIsNone(entry.get(PROP_CREATED))
        self.assertIsNone(entry.get(PROP_MODIFIED))
        last_time = entry.get("Last Push Time")

        result = self.run_cli(cmd_env, base_cmd + f"act push sync '{integ_name}' '{push_name2}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Synchronized push '{push_name2}'", result.out())

        result = self.run_cli(cmd_env, base_cmd + f"act push ls '{integ_name}' --format json --show-times")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("action-push")
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
        result = self.run_cli(cmd_env, base_cmd + f"act push get {integ_name} {push_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc2}", result.out())
        self.assertIn(f"Projects: {proj_name2}", result.out())

        # check the tags in more detail
        result = self.run_cli(cmd_env, list_cmd + "-f json")
        self.assertResultSuccess(result)
        pushes = eval(result.out()).get("action-push")
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
        result = self.run_cli(cmd_env, base_cmd + f"act push tasks '{integ_name}' '{push_name2}' -f json")
        self.assertResultSuccess(result)
        tasks = eval(result.out()).get("action-push-task")
        self.assertGreaterEqual(len(tasks), 1)
        self.assertEqual(1, len(find_by_prop(tasks, "Reason", "push created")))
        entry = tasks[0]
        self.assertIsNotNone(entry.get("Reason"))
        self.assertIsNotNone(entry.get("State"))
        self.assertIsNotNone(entry.get("Status Info"))

        ########################
        # delete the push
        del_cmd = base_cmd + f"act push del '{integ_name}' '{push_name2}' -y"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultSuccess(result)
        self.assertIn("Deleted", result.out())

        # idempotent
        no_push_msg = f"Push action '{push_name2}' not found in integration '{integ_name}'"
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
            base_cmd + f"act push set '{integ_name}' '{push_name1}' --service '{service}' "
            f"--region {region}"
        )
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created push", result.out())

        # do a get to verify the values
        result = self.run_cli(cmd_env, base_cmd + f"act push get '{integ_name}' '{push_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Region: {region}", result.out())
        self.assertIn(f"Service: {service}", result.out())
        self.assertIn(f"Resource: {default_resource}", result.out())

        # delete this push
        result = self.run_cli(cmd_env, base_cmd + f"act push del '{integ_name}' '{push_name1}' -y")
        self.assertResultSuccess(result)

        ########################
        # invalid region
        bad_reg_cmd = base_cmd + f"act push set '{integ_name}' '{push_name2}' --region not-a-region"
        result = self.run_cli(cmd_env, bad_reg_cmd)
        self.assertResultError(result, "isn't a valid value for '--region <region>'")

        ########################
        # no project found
        bogus_project = "this-proj-dne"
        bad_proj_cmd = base_cmd + f"act push set '{integ_name}' '{push_name2}' --project {bogus_project}"
        result = self.run_cli(cmd_env, bad_proj_cmd)
        self.assertResultError(result, f"Project '{bogus_project}' not found")

        ########################
        # invalid tag formats
        pre_tag_cmd = base_cmd + f"act push set '{integ_name}' '{push_name2}' "
        result = self.run_cli(cmd_env, pre_tag_cmd + "--tag foo")
        self.assertResultError(result, "Use a ':' to separate the environment and tag names")

        result = self.run_cli(cmd_env, pre_tag_cmd + "--tag sna:foo:bar")
        self.assertResultError(result, "Can only have one ':' to separate the environment and tag names")

        ########################
        # error out for invalid push name
        result = self.run_cli(cmd_env, base_cmd + f"act push task '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act push get '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act push sync '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        ########################
        # error out for bad integration name
        no_integration_msg = f"Integration '{bad_int_name}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"act p l '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p get '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p set '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p sync '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p task '{bad_int_name}' '{push_name1}' -v")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p del '{bad_int_name}' '{push_name1}' -y")
        self.assertResultError(result, no_integration_msg)

        # cleanup
        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)
        self.delete_environment(cmd_env, env_name1)
        self.delete_environment(cmd_env, env_name2)

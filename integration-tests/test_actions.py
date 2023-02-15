import datetime
import os
import time
import unittest
import subprocess

from typing import List
from typing import Optional

from testcase import TestCase
from testcase import PROP_CREATED
from testcase import PROP_NAME
from testcase import PROP_MODIFIED
from testcase import PROP_VALUE
from testcase import find_by_prop
from testcase import missing_any

CT_PUSH_INTEG_NAME = "CLOUDTRUTH_TEST_PUSH_INTEGRATION_NAME"
CT_PUSH_BAD_INT_NAME = "CLOUDTRUTH_TEST_PUSH_BAD_INTEGRATION_NAME"
CT_PUSH_RUN = [
    CT_PUSH_INTEG_NAME,
    CT_PUSH_BAD_INT_NAME,
]

CT_IMPORT_INTEG_NAME = "CLOUDTRUTH_TEST_IMPORT_INTEGRATION_NAME"
CT_IMPORT_BAD_INT_NAME = "CLOUDTRUTH_TEST_IMPORT_BAD_INTEGRATION_NAME"
CT_IMPORT_RUN = [
    CT_IMPORT_INTEG_NAME,
    CT_IMPORT_BAD_INT_NAME,
]

CT_COMP_INTEG_NAME = "CLOUDTRUTH_TEST_COMPLETE_INTEGRATION_NAME"
CT_COMP_RUN = [
    CT_COMP_INTEG_NAME,
]


class TestActions(TestCase):
    def __init__(self, *args, **kwargs):
        self._pushes = None
        self._pulls = None
        super().__init__(*args, **kwargs)

    def setUp(self) -> None:
        self._pushes = list()
        self._pulls = list()
        super().setUp()

    def tearDown(self) -> None:
        # delete any possibly lingering pushes
        for entry in self._pushes:
            cmd = self._base_cmd + f'act push del -i "{entry[0]}" "{entry[1]}" -y'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # delete any possibly lingering pulls
        for entry in self._pulls:
            cmd = self._base_cmd + f'act import del -i "{entry[0]}" "{entry[1]}" -y'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        super().tearDown()

    @unittest.skipIf(missing_any(CT_PUSH_RUN), "Need all CT_PUSH_RUN parameters")
    def test_action_push_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        integ_name = os.environ.get(CT_PUSH_INTEG_NAME)
        bad_int_name = os.environ.get(CT_PUSH_BAD_INT_NAME)
        set_cmd = base_cmd + "actions push set "

        ########################
        # create a couple projects
        proj_name1 = self.make_name("push-proj1")
        self.create_project(cmd_env, proj_name1)
        proj_name2 = self.make_name("push-proj2")
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
        push_name1 = self.make_name("mypush")
        desc1 = "original comment"
        resource1 = "/{{ environment }}/{{ project }}/{{ parameter }}"
        self._pushes.append((integ_name, push_name1))
        tag1 = f"{env_name1}:{env1_tag1}"
        cmd = (
            set_cmd + f"{push_name1} --integration '{integ_name}' -d '{desc1}' "
            f"--resource '{resource1}' --project '{proj_name1}' --tag '{tag1}'"
        )
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created", result.out())

        ########################
        # check it was created
        list_cmd = base_cmd + f"action push list -i '{integ_name}' "
        pushes = self.get_cli_entries(cmd_env, list_cmd + "-f json", "action-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name1)[0]
        self.assertEqual(entry.get("Projects"), proj_name1)
        self.assertEqual(entry.get("Tags"), f"{tag1}")
        self.assertIsNone(entry.get("Integration"))

        # check the right values were set
        result = self.run_cli(cmd_env, base_cmd + f"act push get -i {integ_name} {push_name1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name1}", result.out())
        self.assertIn(f"Resource: {resource1}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Projects: {proj_name1}", result.out())
        self.assertIn(f"Tags: {tag1}", result.out())
        self.assertIn(f"Region: {default_region}", result.out())
        self.assertIn(f"Service: {default_service}", result.out())
        self.assertIn(f"Integration: {integ_name}", result.out())

        # rename push, change resource, add another project, and another tag
        push_name2 = self.make_name("updatedpush")
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
        pushes = self.get_cli_entries(cmd_env, list_cmd + "-f json", "action-push")
        # original name does not exist
        self.assertEqual(0, len(find_by_prop(pushes, PROP_NAME, push_name1)))
        # check the updated entry
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertEqual(entry.get("Projects"), f"{proj_name1}, {proj_name2}")
        entry_tags = entry.get("Tags")
        self.assertIn(tag1, entry_tags)
        self.assertIn(tag2, entry_tags)
        self.assertEqual(len(entry_tags.split(" ")), 2)

        # check the right values were updated (no integration name specified)
        result = self.run_cli(cmd_env, base_cmd + f"ac push get {push_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Projects: {proj_name1}, {proj_name2}", result.out())
        self.assertIn(f"Tags: {tag2}, {tag1}", result.out())
        self.assertIn(f"Integration: {integ_name}", result.out())

        # list without specifying the integration...
        cmd = base_cmd + "act push ls --format json"
        pushes = self.get_cli_entries(cmd_env, cmd, "action-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertEqual(entry.get("Integration"), integ_name)
        self.assertIsNone(entry.get(PROP_CREATED))
        self.assertIsNone(entry.get(PROP_MODIFIED))
        last_time = entry.get("Last Push Time")

        result = self.run_cli(cmd_env, base_cmd + f"act push sync '{push_name2}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Synchronized push '{push_name2}'", result.out())

        cmd = base_cmd + f"act push ls -i '{integ_name}' --format json --show-times"
        pushes = self.get_cli_entries(cmd_env, cmd, "action-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        self.assertIsNone(entry.get("Integration"))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        self.assertNotEqual(entry.get("Last Push Time"), last_time)

        # change the description, remove a project, and play with tags
        desc2 = "Updated description"
        self._pushes.append((integ_name, push_name2))
        tag3 = f"{env_name1}:{env1_tag2}"
        tag4 = f"{env_name2}:{env2_tag2}"
        cmd = (
            set_cmd + f"'{push_name2}' -d '{desc2}' --no-project '{proj_name1}' --no-tag '{tag1}' "
            f"--tag '{tag3}' --tag '{tag4}' "
        )
        confused_msg = "Multiple tags from the same environment in the same push action require using `{{ tag }}`"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, confused_msg)

        # so, change the resource string, too
        resource3 = f"{resource2}/{{{{ tag }}}}"
        cmd += f"--resource '{resource3}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Updated", result.out())

        # check the right values were updated
        result = self.run_cli(cmd_env, base_cmd + f"act push get -i {integ_name} {push_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {push_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc2}", result.out())
        self.assertIn(f"Projects: {proj_name2}", result.out())

        # check the tags in more detail
        pushes = self.get_cli_entries(cmd_env, list_cmd + "-f json", "action-push")
        entry = find_by_prop(pushes, PROP_NAME, push_name2)[0]
        entry_tags = entry.get("Tags")
        self.assertNotIn(tag1, entry_tags)
        self.assertIn(tag2, entry_tags)
        self.assertIn(tag3, entry_tags)
        self.assertIn(tag4, entry_tags)
        self.assertEqual(len(entry_tags.split(" ")), 3)

        ########################
        # task list
        cmd = base_cmd + f"act push tasks '{push_name2}' -f json"
        tasks = self.get_cli_entries(cmd_env, cmd, "action-push-task")
        self.assertGreaterEqual(len(tasks), 1)
        self.assertEqual(1, len(find_by_prop(tasks, "Reason", "push created")))
        entry = tasks[0]
        self.assertIsNotNone(entry.get("Reason"))
        self.assertIsNotNone(entry.get("State"))
        self.assertIsNotNone(entry.get("Status Info"))
        # task_name = entry.get("Reason")

        ########################
        # task step list
        cmd = base_cmd + f"act push steps '{push_name2}' -f json --show-times"
        steps = self.get_cli_entries(cmd_env, cmd, "action-push-task-step")
        # TODO: why no steps???
        self.assertIsNotNone(steps)
        """
        self.assertGreaterEqual(len(steps), 1)
        entry = steps[0]
        self.assertEqual(entry.get("Task"), task_name)
        self.assertIsNotNone(entry.get("Venue"))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        """

        ########################
        # delete the push
        del_cmd = base_cmd + f"act push del '{push_name2}' -y"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultSuccess(result)
        self.assertIn("Deleted", result.out())

        # idempotent
        no_push_msg = f"Push action '{push_name2}' not found in integration '{integ_name}'"
        result = self.run_cli(cmd_env, del_cmd + f" -i {integ_name}")
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
        cmd = base_cmd + f"act push set -i '{integ_name}' '{push_name1}' --service '{service}' " f"--region {region}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created push", result.out())

        # do a get to verify the values
        result = self.run_cli(cmd_env, base_cmd + f"act push get -i '{integ_name}' '{push_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Region: {region}", result.out())
        self.assertIn(f"Service: {service}", result.out())
        self.assertIn(f"Resource: {default_resource}", result.out())

        # delete this push
        result = self.run_cli(cmd_env, base_cmd + f"act push del -i '{integ_name}' '{push_name1}' -y")
        self.assertResultSuccess(result)

        ########################
        # invalid region
        bad_reg_cmd = base_cmd + f"act push set -i '{integ_name}' '{push_name2}' --region not-a-region"
        result = self.run_cli(cmd_env, bad_reg_cmd)
        self.assertResultError(result, "isn't a valid value for '--region <region>'")

        ########################
        # no project found
        bogus_project = "this-proj-dne"
        bad_proj_cmd = base_cmd + f"act push set -i '{integ_name}' '{push_name2}' --project {bogus_project}"
        result = self.run_cli(cmd_env, bad_proj_cmd)
        self.assertResultError(result, f"Project '{bogus_project}' not found")

        ########################
        # invalid tag formats
        pre_tag_cmd = base_cmd + f"act push set -i '{integ_name}' '{push_name2}' "
        result = self.run_cli(cmd_env, pre_tag_cmd + "--tag foo")
        self.assertResultError(result, "Use a ':' to separate the environment and tag names")

        result = self.run_cli(cmd_env, pre_tag_cmd + "--tag sna:foo:bar")
        self.assertResultError(result, "Can only have one ':' to separate the environment and tag names")

        ########################
        # cannot create without an --integration
        result = self.run_cli(cmd_env, set_cmd + f"{push_name2}")
        self.assertResultError(result, "Must specify an integration on create")

        ########################
        # error out for invalid push name
        result = self.run_cli(cmd_env, base_cmd + f"act push task -i '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act push step -i '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act push get -i '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act push sync -i '{integ_name}' '{push_name2}'")
        self.assertResultError(result, no_push_msg)

        ########################
        # error out for invalid push name (without an integration name)
        no_push_msg2 = f"Push action '{push_name2}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"act push task '{push_name2}'")
        self.assertResultError(result, no_push_msg2)

        result = self.run_cli(cmd_env, base_cmd + f"act push step '{push_name2}'")
        self.assertResultError(result, no_push_msg2)

        result = self.run_cli(cmd_env, base_cmd + f"act push get '{push_name2}'")
        self.assertResultError(result, no_push_msg2)

        result = self.run_cli(cmd_env, base_cmd + f"act push sync '{push_name2}'")
        self.assertResultError(result, no_push_msg2)

        ########################
        # error out for bad integration name
        no_integration_msg = f"Integration '{bad_int_name}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"act p l -i '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p get -i '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p set -i '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p sync -i '{bad_int_name}' '{push_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p task -i '{bad_int_name}' '{push_name1}' -v")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p step -i '{bad_int_name}' '{push_name1}' -v")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act p del -i '{bad_int_name}' '{push_name1}' -y")
        self.assertResultError(result, no_integration_msg)

        # cleanup
        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)
        self.delete_environment(cmd_env, env_name1)
        self.delete_environment(cmd_env, env_name2)

    @unittest.skipIf(missing_any(CT_IMPORT_RUN), "Need all CT_IMPORT_RUN parameters")
    def test_action_import_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        integ_name = os.environ.get(CT_IMPORT_INTEG_NAME)
        bad_int_name = os.environ.get(CT_IMPORT_BAD_INT_NAME)
        set_cmd = base_cmd + "actions import set "

        ########################
        # create the import -- use a bogus resource string the would not exist
        default_service = "ssm"
        default_region = "us-east-1"
        import_name1 = self.make_name("myimport")
        desc1 = "original comment"
        resource1 = "/bogus_resource_path/{{ project }}/{{ environment }}/{{ parameter }}"
        self._pulls.append((integ_name, import_name1))
        cmd = set_cmd + f"{import_name1} --integration '{integ_name}' -d '{desc1}' " f"--resource '{resource1}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created", result.out())

        ########################
        # check it was created
        list_cmd = base_cmd + f"action imports list -i '{integ_name}' "
        imports = self.get_cli_entries(cmd_env, list_cmd + "-f json", "action-import")
        entry = find_by_prop(imports, PROP_NAME, import_name1)[0]
        self.assertEqual(entry.get("Service"), default_service)
        self.assertNotIn("dry-run", entry.get("Flags"))
        self.assertIsNone(entry.get("Integration"))

        # check the right values were set
        result = self.run_cli(cmd_env, base_cmd + f"act imp get -i {integ_name} {import_name1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {import_name1}", result.out())
        self.assertIn(f"Resource: {resource1}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Region: {default_region}", result.out())
        self.assertIn(f"Service: {default_service}", result.out())
        self.assertIn(f"Integration: {integ_name}", result.out())
        self.assertIn("Dry Run: false", result.out())
        self.assertIn("Flags: none", result.out())

        # rename import, change resource, add another project, and another tag
        import_name2 = self.make_name("updatedimp")
        resource2 = "/another_path_should_not_exist/{{ project }}/{{ parameter }}/{{ environment }}"
        self._pulls.append((integ_name, import_name2))
        cmd = set_cmd + f"'{import_name1}' --resource '{resource2}' -r '{import_name2}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Updated", result.out())

        # check we have one entry with updated values
        imports = self.get_cli_entries(cmd_env, list_cmd + "-f json", "action-import")
        # original name does not exist
        self.assertEqual(0, len(find_by_prop(imports, PROP_NAME, import_name1)))
        # check the updated entry
        # TODO: there's more than 1 due to a server bug!
        # self.assertEqual(1, len(find_by_prop(imports, PROP_NAME, import_name2)))

        # check the right values were updated (no integration name specified)
        result = self.run_cli(cmd_env, base_cmd + f"ac im get {import_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {import_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc1}", result.out())
        self.assertIn(f"Region: {default_region}", result.out())
        self.assertIn(f"Service: {default_service}", result.out())
        self.assertIn(f"Integration: {integ_name}", result.out())

        # list without specifying the integration...
        cmd = base_cmd + "act import ls --format json"
        imports = self.get_cli_entries(cmd_env, cmd, "action-import")
        entry = find_by_prop(imports, PROP_NAME, import_name2)[0]
        self.assertEqual(entry.get("Integration"), integ_name)
        self.assertIsNone(entry.get(PROP_CREATED))
        self.assertIsNone(entry.get(PROP_MODIFIED))
        last_time = entry.get("Last Import Time")

        result = self.run_cli(cmd_env, base_cmd + f"act import sync '{import_name2}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Synchronized import '{import_name2}'", result.out())

        cmd = base_cmd + f"act import ls -i '{integ_name}' --format json --show-times"
        imports = self.get_cli_entries(cmd_env, cmd, "action-import")
        entry = find_by_prop(imports, PROP_NAME, import_name2)[0]
        self.assertIsNone(entry.get("Integration"))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        self.assertNotEqual(entry.get("Last Import Time"), last_time)

        # change the description, and set dry-run
        desc2 = "Updated description"
        result = self.run_cli(cmd_env, set_cmd + f"'{import_name2}' -d '{desc2}' --dry-run")
        self.assertResultSuccess(result)
        self.assertIn("Updated", result.out())

        # check the right values were updated
        result = self.run_cli(cmd_env, base_cmd + f"act imports get -i {integ_name} {import_name2}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {import_name2}", result.out())
        self.assertIn(f"Resource: {resource2}", result.out())
        self.assertIn(f"Description: {desc2}", result.out())
        self.assertIn("Dry Run: true", result.out())
        self.assertIn("Flags: dry-run", result.out())

        ########################
        # task list
        cmd = base_cmd + f"act import tasks '{import_name2}' -f json"
        tasks = self.get_cli_entries(cmd_env, cmd, "action-import-task")
        self.assertGreaterEqual(len(tasks), 1)
        # TODO: there's more than 1 due to a server bug!
        # self.assertEqual(1, len(find_by_prop(tasks, "Reason", "pull created")))
        entry = tasks[0]
        self.assertIsNotNone(entry.get("Reason"))
        self.assertIsNotNone(entry.get("State"))
        self.assertIsNotNone(entry.get("Status Info"))
        # task_name = entry.get("Reason")

        ########################
        # task step list
        cmd = base_cmd + f"act import steps '{import_name2}' -f json --show-times"
        steps = self.get_cli_entries(cmd_env, cmd, "action-import-task-step")
        # TODO: why no steps???
        self.assertIsNotNone(steps)
        """
        self.assertGreaterEqual(len(steps), 1)
        entry = steps[0]
        self.assertEqual(entry.get("Task"), task_name)
        self.assertIsNotNone(entry.get("Venue"))
        self.assertIsNotNone(entry.get(PROP_CREATED))
        self.assertIsNotNone(entry.get(PROP_MODIFIED))
        """

        ########################
        # delete the import
        del_cmd = base_cmd + f"act import del '{import_name2}' -y"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultSuccess(result)
        self.assertIn("Deleted", result.out())

        # idempotent
        no_import_msg = f"Import action '{import_name2}' not found in integration '{integ_name}'"
        result = self.run_cli(cmd_env, del_cmd + f" -i {integ_name}")
        self.assertResultWarning(result, no_import_msg)

        # make sure it is gone
        result = self.run_cli(cmd_env, list_cmd + "-f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{import_name1},", result.out())
        self.assertNotIn(f"{import_name2},", result.out())

        ########################
        # create another import -- different values:
        #       check default resource,
        #       secretsmanager service (non-default),
        #       different region (non-default)
        #       dry-run (non-default)
        service = "secretsmanager"
        region = "us-west-2"
        cmd = (
            base_cmd + f"act import set -i '{integ_name}' '{import_name1}' --service '{service}' "
            f"--region {region} --dry-run"
        )
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created import", result.out())

        # do a get to verify the values --
        result = self.run_cli(cmd_env, base_cmd + f"act imp get -i '{integ_name}' '{import_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Region: {region}", result.out())
        self.assertIn(f"Service: {service}", result.out())
        self.assertIn("Dry Run: true", result.out())
        self.assertIn("Flags: dry-run", result.out())

        # delete this import
        result = self.run_cli(cmd_env, base_cmd + f"act import del -i '{integ_name}' '{import_name1}' -y")
        self.assertResultSuccess(result)

        ########################
        # invalid region
        bad_reg_cmd = base_cmd + f"act imp set -i '{integ_name}' '{import_name2}' --region not-a-region"
        result = self.run_cli(cmd_env, bad_reg_cmd)
        self.assertResultError(result, "isn't a valid value for '--region <region>'")

        ########################
        # cannot create without an --integration
        result = self.run_cli(cmd_env, set_cmd + f"{import_name2}")
        self.assertResultError(result, "Must specify an integration on create")

        ########################
        # error out for invalid import name
        result = self.run_cli(cmd_env, base_cmd + f"act import task -i '{integ_name}' '{import_name2}'")
        self.assertResultError(result, no_import_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act import step -i '{integ_name}' '{import_name2}'")
        self.assertResultError(result, no_import_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act import get -i '{integ_name}' '{import_name2}'")
        self.assertResultError(result, no_import_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act import syn -i '{integ_name}' '{import_name2}'")
        self.assertResultError(result, no_import_msg)

        ########################
        # error out for invalid import name (without an integration name)
        no_import_msg2 = f"Import action '{import_name2}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"act im task '{import_name2}'")
        self.assertResultError(result, no_import_msg2)

        result = self.run_cli(cmd_env, base_cmd + f"act im step '{import_name2}'")
        self.assertResultError(result, no_import_msg2)

        result = self.run_cli(cmd_env, base_cmd + f"act im get '{import_name2}'")
        self.assertResultError(result, no_import_msg2)

        result = self.run_cli(cmd_env, base_cmd + f"act im sy '{import_name2}'")
        self.assertResultError(result, no_import_msg2)

        ########################
        # error out for bad integration name
        no_integration_msg = f"Integration '{bad_int_name}' not found"
        result = self.run_cli(cmd_env, base_cmd + f"act i l -i '{bad_int_name}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act i get -i '{bad_int_name}' '{import_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act i set -i '{bad_int_name}' '{import_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act i sync -i '{bad_int_name}' '{import_name1}'")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act i task -i '{bad_int_name}' '{import_name1}' -v")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act i step -i '{bad_int_name}' '{import_name1}' -v")
        self.assertResultError(result, no_integration_msg)

        result = self.run_cli(cmd_env, base_cmd + f"act i del -i '{bad_int_name}' '{import_name1}' -y")
        self.assertResultError(result, no_integration_msg)

    def get_param_pushes(
        self,
        cmd_env,
        proj_name: str,
        env_name: Optional[str] = None,
        param_name: Optional[str] = None,
    ) -> List[dict]:
        push_list_cmd = self._base_cmd + f"--project '{proj_name}' "
        if env_name:
            push_list_cmd += f"--env '{env_name}' "
        push_list_cmd += "param push "
        if param_name:
            push_list_cmd += f"'{param_name}' "
        push_list_cmd += "-f json --show-times"
        return self.get_cli_entries(cmd_env, push_list_cmd, "parameter-push-task-step")

    def success_with(self, cmd_env, command: str, expected: str) -> bool:
        result = self.run_cli(cmd_env, command)
        self.assertResultSuccess(result)
        return expected in result.out()

    def waitFor(
        self,
        func,
        timeout_minutes: int = 3,
        sleep_seconds: int = 3,
    ) -> bool:
        completed = False
        start_time = datetime.datetime.now()
        max_time = datetime.timedelta(minutes=timeout_minutes)
        end_time = start_time + max_time
        while datetime.datetime.now() < end_time and not completed:
            time.sleep(sleep_seconds)  # don't hammer the server too hard
            completed = func()

        self.assertTrue(completed, "Timed out without succeeding")

    @unittest.skipIf(missing_any(CT_COMP_RUN), "Need all CT_COMP_RUN parameters")
    def test_action_complete(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        push_cmd = base_cmd + "act push "
        imp_cmd = base_cmd + "act imp "

        # TODO: other tests include:
        #  1. secrets -- how are they handled (saved in plaintest to SSM? if existing param, not a secret)
        #  2. unsaved parameters in project -- removed or retained?
        integ_name = os.environ.get(CT_COMP_INTEG_NAME)

        ########################
        # create a couple environments
        env_name_a = self.make_name("act-comp-a")
        self.create_environment(cmd_env, env_name_a)
        env_name_b = self.make_name("act-comp-b")
        self.create_environment(cmd_env, env_name_b)

        ########################
        # define a couple projects, parameters, and values
        proj_name1 = self.make_name("comp-army")
        param11 = "soldier"
        value11a = "private"
        value11b = "infantryman"
        param21 = "officer"
        value21a = "captain"
        value21b = "engineer"

        proj_name2 = self.make_name("comp-airforce")
        param12 = "officer"
        value12a = "navigator"
        value12b = "major"
        param22 = "airman"
        value22a = "mechanic"
        value22b = "techsergeant"

        def create_project1():
            ########################
            # create parameters and values
            self.create_project(cmd_env, proj_name1)
            self.set_param(cmd_env, proj_name1, param11, value=value11a, env=env_name_a)
            self.set_param(cmd_env, proj_name1, param11, value=value11b, env=env_name_b)
            self.set_param(cmd_env, proj_name1, param21, value=value21a, env=env_name_a)
            self.set_param(cmd_env, proj_name1, param21, value=value21b, env=env_name_b)

        def create_project2():
            self.create_project(cmd_env, proj_name2)
            self.set_param(cmd_env, proj_name2, param12, value=value12a, env=env_name_a)
            self.set_param(cmd_env, proj_name2, param12, value=value12b, env=env_name_b)
            self.set_param(cmd_env, proj_name2, param22, value=value22a, env=env_name_a)
            self.set_param(cmd_env, proj_name2, param22, value=value22b, env=env_name_b)

        def validate_project1a():
            output = self.list_params(cmd_env, proj_name1, env=env_name_a, fmt="json")
            proj1a_params = eval(output.out()).get("parameter")
            entry = find_by_prop(proj1a_params, PROP_NAME, param11)[0]
            self.assertEqual(entry.get(PROP_VALUE), value11a)
            entry = find_by_prop(proj1a_params, PROP_NAME, param21)[0]
            self.assertEqual(entry.get(PROP_VALUE), value21a)

        def validated_project1b():
            output = self.list_params(cmd_env, proj_name1, env=env_name_b, fmt="json")
            proj1a_params = eval(output.out()).get("parameter")
            entry = find_by_prop(proj1a_params, PROP_NAME, param11)[0]
            self.assertEqual(entry.get(PROP_VALUE), value11b)
            entry = find_by_prop(proj1a_params, PROP_NAME, param21)[0]
            self.assertEqual(entry.get(PROP_VALUE), value21b)

        def validate_project2a():
            output = self.list_params(cmd_env, proj_name2, env=env_name_a, fmt="json")
            proj1a_params = eval(output.out()).get("parameter")
            entry = find_by_prop(proj1a_params, PROP_NAME, param12)[0]
            self.assertEqual(entry.get(PROP_VALUE), value12a)
            entry = find_by_prop(proj1a_params, PROP_NAME, param22)[0]
            self.assertEqual(entry.get(PROP_VALUE), value22a)

        def validate_project2b():
            output = self.list_params(cmd_env, proj_name2, env=env_name_b, fmt="json")
            proj1a_params = eval(output.out()).get("parameter")
            entry = find_by_prop(proj1a_params, PROP_NAME, param12)[0]
            self.assertEqual(entry.get(PROP_VALUE), value12b)
            entry = find_by_prop(proj1a_params, PROP_NAME, param22)[0]
            self.assertEqual(entry.get(PROP_VALUE), value22b)

        # create and validate the projects here
        create_project1()
        create_project2()
        validate_project1a()
        validated_project1b()
        validate_project2a()
        validate_project2b()

        # create tags on the environments
        env_a_tag_name = "war"
        env_b_tag_name = "peace"
        tag_a = f"{env_name_a}:{env_a_tag_name}"
        tag_b = f"{env_name_b}:{env_b_tag_name}"
        self.create_env_tag(cmd_env, env_name_a, env_a_tag_name)
        self.create_env_tag(cmd_env, env_name_b, env_b_tag_name)

        ########################
        # create the push of both projects -- the path includes a test-specific
        prefix = self.make_name("action-complete")
        resource = f"/{prefix}" + "/{{ environment }}/{{ project }}/{{ parameter }}"
        push_name = self.make_name("comp-push")
        self._pushes.append((integ_name, push_name))
        create_push_cmd = (
            push_cmd + f"set {push_name} --integration '{integ_name}' "
            f"--resource '{resource}' --project '{proj_name1}' --project '{proj_name2}' "
            f"--tag '{tag_a}' --tag '{tag_b}' "
        )
        result = self.run_cli(cmd_env, create_push_cmd)
        self.assertResultSuccess(result)

        # wait for the push to complete
        def push_success() -> bool:
            get_push_cmd = push_cmd + f"get '{push_name}' -i '{integ_name}'"
            return self.success_with(cmd_env, get_push_cmd, "State: success")

        self.waitFor(push_success)

        # check that we have a step for each
        cmd = push_cmd + f"steps '{push_name}' -i '{integ_name}' -f json"
        push_steps = self.get_cli_entries(cmd_env, cmd, "action-push-task-step")
        push_step_len = len(push_steps)
        self.assertEqual(push_step_len, 8)
        self.assertEqual(len(find_by_prop(push_steps, "Task", "push created")), push_step_len)

        ########################
        # look at push tasks from the parameter standpoint -- one for each variable
        param_pushes = self.get_param_pushes(cmd_env, proj_name1, env_name=env_name_a)
        self.assertEqual(len(param_pushes), 2)
        param_pushes = self.get_param_pushes(cmd_env, proj_name1, env_name=env_name_b)
        self.assertEqual(len(param_pushes), 2)
        param_pushes = self.get_param_pushes(cmd_env, proj_name2, env_name=env_name_a)
        self.assertEqual(len(param_pushes), 2)
        param_pushes = self.get_param_pushes(cmd_env, proj_name2, env_name=env_name_b)
        self.assertEqual(len(param_pushes), 2)

        ########################
        # both projects listed in push
        cmd = push_cmd + f"list -i '{integ_name}' -f json"
        entries = self.get_cli_entries(cmd_env, cmd, "action-push")
        push_entry = find_by_prop(entries, PROP_NAME, push_name)[0]
        self.assertIn(proj_name1, push_entry.get("Projects"))
        self.assertIn(proj_name2, push_entry.get("Projects"))

        ########################
        # delete/modify project data -- no tags harmed

        # delete one project and one environment (should be restored by import)
        self.delete_project(cmd_env, proj_name2)

        # change values in the other project... should be overwritten, or restored
        self.set_param(cmd_env, proj_name1, param11, value="marine", env=env_name_a)
        self.delete_param(cmd_env, proj_name1, param12)

        ########################
        # create the import
        import_name = self.make_name("my-comp-imp")
        self._pulls.append((integ_name, import_name))
        create_import_cmd = imp_cmd + f"set '{import_name}' --integration '{integ_name}' --resource '{resource}'"
        result = self.run_cli(cmd_env, create_import_cmd)
        self.assertResultSuccess(result)

        # wait for the pull to complete
        def pull_success() -> bool:
            get_import_cmd = imp_cmd + f"get '{import_name}' -i '{integ_name}'"
            return self.success_with(cmd_env, get_import_cmd, "State: success")

        self.waitFor(pull_success)

        cmd = imp_cmd + f"st '{import_name}' -i '{integ_name}' -f json"
        import_steps = self.get_cli_entries(cmd_env, cmd, "action-import-task-step")
        import_step_len = len(import_steps)
        self.assertGreaterEqual(import_step_len, 8)
        self.assertEqual(len(find_by_prop(import_steps, "Task", "pull created")), import_step_len)

        # verify everything was put back to the original state
        validate_project1a()
        validated_project1b()
        validate_project2a()
        validate_project2b()

        ########################
        # project2 was restored, it does not have the same id, so is no longer associated
        cmd = push_cmd + f"list -i '{integ_name}' -f json"
        entries = self.get_cli_entries(cmd_env, cmd, "action-push")
        push_entry = find_by_prop(entries, PROP_NAME, push_name)[0]
        self.assertEqual(push_entry.get("Projects"), proj_name1)

        ########################
        # delete an environment -- deletes tags and causes push update
        self.delete_environment(cmd_env, env_name_b)

        # wait for push update
        def more_push_steps() -> bool:
            push_step_cmd = push_cmd + f"task-steps {push_name} -f json"
            more_steps = self.get_cli_entries(cmd_env, push_step_cmd, "action-push-task-step")
            return len(more_steps) > push_step_len and push_success()

        self.waitFor(more_push_steps)

        ########################
        # re-establish project2 push connection
        entries = self.get_cli_entries(cmd_env, push_cmd + f"st {push_name} -f json", "action-push-task-step")
        push_step_len = len(entries)

        # update the tag
        result = self.run_cli(cmd_env, base_cmd + f"env tag set {env_name_a} {env_a_tag_name} --current")
        self.assertResultSuccess(result)

        # wait for the tag update to update the push tasks
        self.waitFor(more_push_steps)
        entries = self.get_cli_entries(cmd_env, push_cmd + f"st {push_name} -f json", "action-push-task-step")
        push_step_len = len(entries)

        # assign the project and environment/tag
        cmd = push_cmd + f"set {push_name} -i '{integ_name}' --project {proj_name2} --tag {tag_a}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.waitFor(more_push_steps)

        ########################
        # change project data again
        self.delete_project(cmd_env, proj_name1)
        self.set_param(cmd_env, proj_name2, param22, "grunt", env=env_name_a)

        ########################
        # import again
        result = self.run_cli(cmd_env, imp_cmd + f"sync '{import_name}'")
        self.assertResultSuccess(result)

        def more_pull_steps() -> bool:
            import_step_cmd = imp_cmd + f"task-steps {import_name} -f json"
            more_steps = self.get_cli_entries(cmd_env, import_step_cmd, "action-import-task-step")
            return len(more_steps) > import_step_len and pull_success()

        self.waitFor(more_pull_steps)

        validate_project1a()
        validate_project2a()
        environments = self.get_cli_entries(cmd_env, base_cmd + "env ls -f json", "environment")
        self.assertEqual(0, len(find_by_prop(environments, PROP_NAME, env_name_b)))

        # cleanup
        self.run_cli(cmd_env, base_cmd + f"action push del '{push_name}' -yi '{integ_name}'")
        self.assertResultSuccess(result)
        self.run_cli(cmd_env, base_cmd + f"action import del '{import_name}' -yi '{integ_name}'")
        self.assertResultSuccess(result)
        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)
        self.delete_environment(cmd_env, env_name_a)
        # NOTE: env_name_b was deleted earlier

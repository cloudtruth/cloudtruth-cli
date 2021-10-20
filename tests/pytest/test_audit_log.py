from typing import Dict
from typing import List
from typing import Optional

from testcase import TestCase
from testcase import find_by_prop

PROP_TYPE = "Type"
PROP_ACTION = "Action"
PROP_NAME = "Object Name"


class TestAuditLogs(TestCase):
    def assertCreateDelete(self, entries):
        # see that we have both create/delete actions
        created = find_by_prop(entries, PROP_ACTION, "create")
        self.assertNotEqual(0, len(created))
        deleted = find_by_prop(entries, PROP_ACTION, "delete")
        self.assertNotEqual(0, len(deleted))

    def audit_entries(
            self,
            cmd_env,
            type_str: Optional[str] = None,
            name: Optional[str] = None,
            action: Optional[str] = None,
            max_entries: Optional[int] = None,
    ) -> List[Dict]:
        cmd = self.get_cli_base_cmd() + "audit-logs ls -f json "
        if type_str:
            cmd += f"-t '{type_str}' "
        if name:
            cmd += f"-n '{name}' "
        if action:
            cmd += f"-a '{action}' "
        if max_entries:
            cmd += f"-m {max_entries}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        if result.out().startswith("No audit log entries"):
            return []
        return eval(result.out()).get("audit-logs")

    def test_audit_logs(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        audit_cmd = base_cmd + "audit "

        # take a summary snapshot
        result = self.run_cli(cmd_env, audit_cmd + "summary")
        self.assertResultSuccess(result)
        orig_summary = result.out()

        # add some things
        proj_name = self.make_name("test-audit")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("aud-env")
        self.create_environment(cmd_env, env_name)
        param1 = "aud-param"
        value1 = "this is the value for the audit log test"
        self.set_param(cmd_env, proj_name, param1, value=value1, env=env_name)
        temp_name = "my-aud-temp"
        body = "# this template has just fixed text"
        self.set_template(cmd_env, proj_name, temp_name, body=body)

        # TODO: update items

        # delete the things
        self.delete_template(cmd_env, proj_name, temp_name)
        self.delete_param(cmd_env, proj_name, param1)
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

        #############################
        # check that we have audit trail entries for each type

        # NOTE: cannot allow all types because of issue with Tag object_type
        entries = self.audit_entries(cmd_env, "parameter")
        filtered = find_by_prop(entries, PROP_TYPE, "parameter")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, param1)
        self.assertCreateDelete(filtered)

        max_entries = 25
        entries = self.audit_entries(cmd_env, "template", temp_name, max_entries=max_entries)
        self.assertLessEqual(len(entries), max_entries)
        filtered = find_by_prop(entries, PROP_TYPE, "template")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, temp_name)
        self.assertCreateDelete(filtered)

        action = "create"
        entries = self.audit_entries(cmd_env, "environment", env_name, action=action)
        filtered = find_by_prop(entries, PROP_TYPE, "environment")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_ACTION, action)
        self.assertEqual(len(entries), len(filtered))

        value_name = f"{param1}:{env_name}"
        max_entries = 5
        entries = self.audit_entries(cmd_env, "value", value_name, max_entries=max_entries)
        filtered = find_by_prop(entries, PROP_TYPE, "value")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, value_name)
        self.assertCreateDelete(filtered)

        #####################################
        # just a basic thing to make sure our filters work
        for obj_type in [
            "aws", "github", "invitation", "membership", "organization", "rule", "push",
            "service-account", "tag",
        ]:
            max_entries = 5
            entries = self.audit_entries(cmd_env, obj_type, max_entries=max_entries)
            filtered = find_by_prop(entries, PROP_TYPE, obj_type)
            self.assertLessEqual(len(entries), max_entries)
            self.assertEqual(len(entries), len(filtered))

        #####################################
        # unfiltered
        entries = self.audit_entries(cmd_env)
        self.assertNotEqual(len(entries), 0)

        # final snapshot
        result = self.run_cli(cmd_env, audit_cmd + "sum")
        self.assertResultSuccess(result)
        final_summary = result.out()

        # compare summaries -- cannot guarantee count has gone up, since pruning is async
        self.assertNotEqual(orig_summary, final_summary)

from typing import Dict
from typing import List
from typing import Optional

from testcase import TestCase

PROP_TYPE = "Type"
PROP_ACTION = "Action"
PROP_NAME = "Object Name"


def find_by_prop(entries: List[Dict], prop_name: str, prop_value: str) -> List[Dict]:
    return [e for e in entries if e.get(prop_name, None) == prop_value]


class TestAuditLogs(TestCase):
    def assertCreateDelete(self, entries):
        # see that we have both create/delete actions
        created = find_by_prop(entries, PROP_ACTION, "create")
        self.assertNotEqual(0, len(created))
        deleted = find_by_prop(entries, PROP_ACTION, "delete")
        self.assertNotEqual(0, len(deleted))

    def audit_entries(self, cmd_env, type_str: Optional[str]) -> List[Dict]:
        cmd = self.get_cli_base_cmd() + "audit-logs ls -f json "
        if type_str:
            cmd += f"-t '{type_str}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
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

        entries = self.audit_entries(cmd_env, "template")
        filtered = find_by_prop(entries, PROP_TYPE, "template")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, temp_name)
        self.assertCreateDelete(filtered)

        entries = self.audit_entries(cmd_env, "environment")
        filtered = find_by_prop(entries, PROP_TYPE, "environment")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, env_name)
        self.assertCreateDelete(filtered)

        entries = self.audit_entries(cmd_env, "parameter")
        filtered = find_by_prop(entries, PROP_TYPE, "parameter")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, f"{param1}")
        self.assertCreateDelete(filtered)

        entries = self.audit_entries(cmd_env, "value")
        filtered = find_by_prop(entries, PROP_TYPE, "value")
        self.assertEqual(len(entries), len(filtered))
        filtered = find_by_prop(entries, PROP_NAME, f"{param1}:{env_name}")
        self.assertCreateDelete(filtered)

        # TODO: test tag, rule

        # final snapshot
        result = self.run_cli(cmd_env, audit_cmd + "sum")
        self.assertResultSuccess(result)
        final_summary = result.out()

        # compare summaries -- cannot guarantee count has gone up, since pruning is async
        self.assertNotEqual(orig_summary, final_summary)

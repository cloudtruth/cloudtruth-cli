import datetime
from typing import Dict
from typing import List
from typing import Optional

from testcase import TestCase, CT_API_KEY
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
            before: Optional[str] = None,
            after: Optional[str] = None,
            username: Optional[str] = None,
    ) -> List[Dict]:
        cmd = self.get_cli_base_cmd() + "audit-logs ls -f json "
        if type_str:
            cmd += f"-t '{type_str}' "
        if name:
            cmd += f"-n '{name}' "
        if action:
            cmd += f"-a '{action}' "
        if max_entries:
            cmd += f"-m {max_entries} "
        if before:
            cmd += f"--before '{before}' "
        if after:
            cmd += f"--after '{after}' "
        if username:
            cmd += f"--user '{username}' "

        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        if result.out().startswith("No audit log entries"):
            return []
        return eval(result.out()).get("audit-logs")

    def test_audit_logs(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        audit_cmd = base_cmd + "audit "

        # create a user, so we can check attribution
        user_name = self.make_name("log-user")
        api_key = self.add_user(cmd_env, user_name, role="admin")
        cmd_env[CT_API_KEY] = api_key

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

        # get a fresh copy that does not have the deleted user API key
        cmd_env = self.get_cmd_env()

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
        # time filtered
        # nothing found for old date
        entries = self.audit_entries(cmd_env, max_entries=0, before="2021-10-31")
        self.assertEqual(0, len(entries))

        # nothing found for a date in the future
        entries = self.audit_entries(cmd_env, max_entries=0, after="2051-10-31")
        self.assertEqual(0, len(entries))

        def get_value(orig_text: str, label: str) -> str:
            return [_ for _ in orig_text.split("\n") if label in _][0].replace(label, "").strip()

        # get the timestamp from the earliest timestamp in the summary
        label = "Earliest record:"
        value = get_value(orig_summary, label).replace("Z", "")
        oldest = datetime.datetime.fromisoformat(value)

        label = "Record count:"
        value = get_value(orig_summary, label)
        total_records = int(value)

        # round the filter up to the next minute
        rounded = oldest + datetime.timedelta(minutes=1)
        before = f"{rounded.year}-{rounded.month}-{rounded.day}T{rounded.hour}:{rounded.minute}:00Z"

        entries = self.audit_entries(cmd_env, before=before, max_entries=0)
        self.assertLessEqual(len(entries), total_records)
        newer = [e for e in entries if e.get("Time") > before]
        self.assertEqual(len(newer), 0)

        # get a latest time... other entries could be added in the interim, so don't lock down the #
        entries = self.audit_entries(cmd_env, max_entries=3)
        after = entries[-1].get("Time")
        entries = self.audit_entries(cmd_env, after=after, max_entries=0)
        self.assertLessEqual(len(entries), total_records)
        older = [e for e in entries if e.get("Time") < after]
        self.assertEqual(len(older), 0)

        #####################################
        # test bad time formats
        after_err = "Invalid '--after' value"
        before_err = "Invalid '--before' value"
        result = self.run_cli(cmd_env, base_cmd + "log ls --before foo")
        self.assertResultError(result, before_err)

        result = self.run_cli(cmd_env, base_cmd + "log ls --after bar")
        self.assertResultError(result, after_err)

        result = self.run_cli(cmd_env, base_cmd + "log ls --after bar --before foo")
        self.assertResultError(result, after_err)
        self.assertIn(before_err, result.err())

        #####################################
        # test user filter
        entries = self.audit_entries(cmd_env, username=user_name)
        self.assertGreaterEqual(len(entries), 8)  # we created/delete 4 objects as this user above
        other_users = [_ for _ in entries if _.get("User") != user_name]
        self.assertEqual(0, len(other_users))

        # error for an unknown user
        alternate_user = "ricardo.multiban"
        result = self.run_cli(cmd_env, base_cmd + f"audit-logs ls --user '{alternate_user}'")
        self.assertResultError(result, f"User '{alternate_user}' not found")

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

        # cleanup
        self.delete_user(cmd_env, user_name)

from typing import Dict

from testcase import TestCase
from testcase import find_by_prop

DEFAULT_ROLE = "viewer"
SERVICE_TYPE = "service"
PROP_NAME = "Name"
PROP_ROLE = "Role"
PROP_DESC = "Description"
PROP_TYPE = "Type"


class TestUsers(TestCase):
    def _get_user_entry(self, cmd_env, user_name: str) -> Dict:
        result = self.run_cli(cmd_env, self._base_cmd + "users ls -v -f json")
        self.assertResultSuccess(result)
        entries = eval(result.out()).get("user")
        return find_by_prop(entries, PROP_NAME, user_name)[0]

    def test_user_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        user_name = self.make_name("test-user-name")
        sub_cmd = base_cmd + "users "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{user_name},", result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {user_name}")
        self.assertResultError(result, f"The user '{user_name}' could not be found")

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --desc \"{orig_desc}\"")
        self.assertResultSuccess(result)
        api_key = result.stdout[1]

        result = self.run_cli(cmd_env, sub_cmd + "list --values --format csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{user_name},{SERVICE_TYPE},{DEFAULT_ROLE},,{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        entry = self._get_user_entry(cmd_env, user_name)
        self.assertEqual(entry.get(PROP_TYPE), SERVICE_TYPE)
        self.assertEqual(entry.get(PROP_ROLE), DEFAULT_ROLE)
        self.assertEqual(entry.get(PROP_DESC), new_desc)

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        # use the new API key -- see we get some environments back
        result = self.run_cli(cmd_env, base_cmd + f"--api-key {api_key} env ls -vf csv")
        self.assertResultSuccess(result)

        # since the default is a 'viewer' role, see that we cannot set ourself to owner
        permission_err = "You do not have permission to perform this action"
        cmd = base_cmd + f"--api-key '{api_key}' user set '{user_name}' --role owner"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, permission_err)

        # update the role
        new_role = "contrib"
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name} --role {new_role}")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated user '{user_name}'", result.out())

        # see the updated role
        entry = self._get_user_entry(cmd_env, user_name)
        self.assertEqual(entry.get(PROP_TYPE), SERVICE_TYPE)
        self.assertEqual(entry.get(PROP_ROLE), new_role)
        self.assertEqual(entry.get(PROP_DESC), new_desc)

        result = self.run_cli(cmd_env, sub_cmd + f"get {user_name}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {user_name}", result.out())
        self.assertIn(f"Role: {new_role}", result.out())
        self.assertIn(f"Description: {new_desc}", result.out())
        self.assertIn(f"Type: {SERVICE_TYPE}", result.out())
        self.assertIn("Created At: ", result.out())
        self.assertIn("Modified At: ", result.out())
        self.assertIn("Last Used At: ", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {user_name}")
        self.assertResultWarning(
            result,
            f"User '{user_name}' not updated: no updated parameters provided",
        )

        # use the new API key -- see we get some environments back
        result = self.run_cli(cmd_env, base_cmd + f"--api-key {api_key} env ls -vf csv")
        self.assertResultSuccess(result)

        # try creating a new owner
        user2_name = self.make_name("test-another")
        cmd = base_cmd + f"--api-key '{api_key}' user set '{user2_name}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultError(result, permission_err)

        # test the list without the values -- check whole line matches
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(user_name, result.stdout)
        self.assertNotIn(user2_name, result.stdout)
        self.assertNotIn(new_desc, result.stdout)

        # get the new users as the current user
        result = self.run_cli(cmd_env, base_cmd + f"--api-key {api_key} users current")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {user_name}", result.out())
        self.assertIn(f"Role: {new_role}", result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At,Last Used At", result.out())
        self.assertIn(user_name, result.out())
        self.assertIn(new_desc, result.out())

        # get the current user
        result = self.run_cli(cmd_env, sub_cmd + "current")
        self.assertResultSuccess(result)
        self.assertIn("Name: ", result.out())
        self.assertIn("Role: ", result.out())
        self.assertIn("Type: service", result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {user_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{user_name},", result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {user_name} --confirm")
        self.assertResultWarning(result, f"User '{user_name}' does not exist")

    def _get_invite_entry(self, cmd_env, email: str) -> Dict:
        result = self.run_cli(cmd_env, self._base_cmd + "users invite ls -v -f json")
        self.assertResultSuccess(result)
        entries = eval(result.out()).get("invitation")
        matches = find_by_prop(entries, "Email", email)
        if len(matches) >= 1:
            return matches[0]
        return None

    def test_user_invitations(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()
        invite_cmd = base_cmd + "user invite "
        def_role = 'viewer'
        curr_user = self.current_username(cmd_env)

        invitee = "ci.invites"
        job_id = self.make_name("")
        if job_id:
            job_id = f"+{job_id}"
        email = invitee + job_id + "@cloudtruth.com"

        # check to make sure the entry does not exist
        entry = self._get_invite_entry(cmd_env, email)
        self.assertIsNone(entry)

        # create an invitation (default role)
        result = self.run_cli(cmd_env, invite_cmd + f"set '{email}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Sent '{email}' invitation as '{def_role}'", result.out())

        # see it shows up
        entry = self._get_invite_entry(cmd_env, email)
        self.assertEqual(entry.get("Role"), def_role)
        self.assertEqual(entry.get("Inviter"), curr_user)
        self.assertEqual(entry.get("State"), "sent")  # TODO: race condition?

        # see warning when nothing changes
        result = self.run_cli(cmd_env, invite_cmd + f"set '{email}'")
        self.assertResultWarning(result, f"Invitation for '{email}' not updated")

        # update the role
        new_role = "contrib"
        result = self.run_cli(cmd_env, invite_cmd + f"set '{email}' --role '{new_role}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated invitation for '{email}'", result.out())

        # see the new role
        entry = self._get_invite_entry(cmd_env, email)
        self.assertEqual(entry.get("Role"), new_role)
        self.assertEqual(entry.get("Inviter"), curr_user)
        self.assertEqual(entry.get("State"), "sent")  # TODO: race condition?

        # resend the invitation
        result = self.run_cli(cmd_env, invite_cmd + f"resend '{email}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Resent invitation for '{email}'", result.out())

        # delete the invitation
        result = self.run_cli(cmd_env, invite_cmd + f"del -y '{email}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Deleted invitation for '{email}'", result.out())

        # idempotent
        result = self.run_cli(cmd_env, invite_cmd + f"del -y '{email}'")
        self.assertResultWarning(result, f"Invitation for '{email}' does not exist")

from typing import Dict

from testcase import TestCase
from testcase import find_by_prop

PROP_NAME = "Name"
PROP_DESC = "Description"
PROP_USERS = "Users"


class TestGroups(TestCase):
    def _get_group_entry(self, cmd_env, group_name: str) -> Dict:
        entries = self.get_cli_entries(cmd_env, self._base_cmd + "groups ls -v -f json", "group")
        return find_by_prop(entries, PROP_NAME, group_name)[0]

    def test_group_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        group_name = self.make_name("group-name")
        sub_cmd = base_cmd + "groups "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{group_name},", result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {group_name}")
        self.assertResultError(result, f"The group '{group_name}' could not be found")

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {group_name} --desc \"{orig_desc}\"")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "list --values --format csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{group_name},{orig_desc}", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {group_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        entry = self._get_group_entry(cmd_env, group_name)
        self.assertEqual(entry.get(PROP_DESC), new_desc)

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {group_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + f"get {group_name}")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {group_name}", result.out())
        self.assertIn(f"Description: {new_desc}", result.out())
        self.assertIn("Created At: ", result.out())
        self.assertIn("Modified At: ", result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At", result.out())
        self.assertIn(group_name, result.out())
        self.assertIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {group_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{group_name},", result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {group_name} --confirm")
        self.assertResultWarning(result, f"Group '{group_name}' does not exist")

    def test_group_pagination(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()
        group_cmd = base_cmd + "group "

        page_size = 2
        group_count = page_size + 1
        group_names = []
        for idx in range(group_count):
            name = f"ci-group+{idx}"
            job_id = self.make_name("")
            if job_id:
                name += f"-{job_id}"
            group_names.append(name)

        for name in group_names:
            result = self.run_cli(cmd_env, group_cmd + f"set {name}")
            self.assertResultSuccess(result)

        invite_list_cmd = group_cmd + "ls"
        self.assertPaginated(cmd_env, invite_list_cmd, "/groups/?", page_size=page_size)

        result = self.run_cli(cmd_env, invite_list_cmd)
        self.assertResultSuccess(result)
        output = result.out()
        for name in group_names:
            self.assertIn(name, output)

        # cleanup
        for name in group_names:
            result = self.run_cli(cmd_env, group_cmd + f"del -y {name}")
            self.assertResultSuccess(result)

    def test_group_users(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        group_name = self.make_name("group-name")
        group_cmd = base_cmd + "groups "
        user_cmd = base_cmd + "users "

        user_count = 3
        user_names = []
        for idx in range(user_count):
            name = f"ci-user+{idx}"
            user_names.append(self.make_name(name))

        # create group
        result = self.run_cli(cmd_env, group_cmd + f"set {group_name}")
        self.assertResultSuccess(result)

        # create users
        for user_name in user_names:
            result = self.run_cli(cmd_env, user_cmd + f"set {user_name}")
            self.assertResultSuccess(result)

        # add users to group
        for user_name in user_names:
            result = self.run_cli(cmd_env, group_cmd + f"set {group_name} --add-user {user_name}")
            self.assertResultSuccess(result)

        # remove users from group
        for user_name in user_names:
            result = self.run_cli(cmd_env, group_cmd + f"set {group_name} --remove-user {user_name}")
            self.assertResultSuccess(result)

        # add all users with one command
        add_all_users_cmd = group_cmd + f"set {group_name} "
        add_all_users_cmd += " ".join(f"--add-user {user_name}" for user_name in user_names)
        result = self.run_cli(cmd_env, add_all_users_cmd)
        self.assertResultSuccess(result)

        # remove all users from group with one command
        rm_all_users_cmd = group_cmd + f"set {group_name} "
        rm_all_users_cmd += " ".join(f"--remove-user {user_name}" for user_name in user_names)
        result = self.run_cli(cmd_env, rm_all_users_cmd)
        self.assertResultSuccess(result)

        # cleanup
        for user_name in user_names:
            result = self.run_cli(cmd_env, user_cmd + f"delete {user_name} --confirm")
            self.assertResultSuccess(result)
        self.run_cli(cmd_env, group_cmd + f"delete {group_name} --confirm")
        self.assertResultSuccess(result)

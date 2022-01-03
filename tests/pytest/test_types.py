from testcase import TestCase
from testcase import find_by_prop
from testcase import TEST_PAGE_SIZE


class TestParameterTypes(TestCase):
    def test_type_basic(self):
        # verify `type_name` does not yet exist
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        type_name = self.make_name("test-type-name")
        sub_cmd = base_cmd + "types "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(type_name, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {type_name} --desc \"{orig_desc}\"")
        self.assertResultSuccess(result)
        entries = self.get_cli_entries(cmd_env, sub_cmd + "ls -f json", "parameter-type")
        entry = find_by_prop(entries, "Name", type_name)[0]
        self.assertEqual(entry.get("Parent"), "string")  # default
        self.assertEqual(entry.get("Description"), orig_desc)

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {type_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name},string,{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {type_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        # rename
        orig_name = type_name
        type_name = self.make_name("test-type-rename")
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{type_name}\"")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated parameter type '{type_name}'", result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {type_name}")
        self.assertResultWarning(
            result,
            f"Parameter type '{type_name}' not updated: nothing to update",
        )

        # test the list without the values
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(type_name, result.out())
        self.assertNotIn(new_desc, result.out())

        # shows create/modified times
        result = self.run_cli(cmd_env, sub_cmd + "list --show-times -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Created At,Modified At", result.out())
        self.assertIn(type_name, result.out())
        self.assertIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {type_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(type_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {type_name} --confirm")
        self.assertResultWarning(result, f"Parameter type '{type_name}' does not exist")

    def test_type_parents(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        type_name1 = self.make_name("type-par-1")
        type_name2 = self.make_name("type-mid-2")
        type_name3 = self.make_name("type-chld-3")
        type_name4 = self.make_name("type-chld-4")

        self.create_type(cmd_env, type_name1)
        self.create_type(cmd_env, type_name2, parent=type_name1)
        self.create_type(cmd_env, type_name3, parent=type_name2)
        self.create_type(cmd_env, type_name4, parent=type_name2)

        # Use csv to validate, since the names may be variable
        result = self.run_cli(cmd_env, base_cmd + "type ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name1},string,", result.out())
        self.assertIn(f"{type_name2},{type_name1},", result.out())
        self.assertIn(f"{type_name3},{type_name2},", result.out())
        self.assertIn(f"{type_name4},{type_name2},", result.out())

        # basic 'tree' test
        result = self.run_cli(cmd_env, base_cmd + "ty tree")
        self.assertResultSuccess(result)
        expected = f"  {type_name1}\n    {type_name2}\n      {type_name3}\n      {type_name4}\n"
        self.assertIn(expected, result.out())

        # TODO: attempt to delete something that is used elsewhere
        '''
        result = self.run_cli(cmd_env, base_cmd + f"type delete '{type_name2}' --confirm")
        self.assertResultError(result, "Cannot remove type because the following type(s) depend on it")
        self.assertIn(type_name3, result.err())
        self.assertIn(type_name4, result.err())
        '''

        # attempt to create without an existing parent
        type_name5 = self.make_name("type-par-5")
        type_name6 = self.make_name("type-par-6")
        result = self.run_cli(cmd_env, base_cmd + f"type set '{type_name5}' --parent '{type_name6}'")
        self.assertResultError(result, f"No parent parameter type '{type_name6}' found")

        # update parent -- success
        result = self.run_cli(cmd_env, base_cmd + f"type set '{type_name4}' --parent '{type_name1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated parameter type '{type_name4}'", result.out())

        result = self.run_cli(cmd_env, base_cmd + "parameter-type ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{type_name4},{type_name1},", result.out())

        # setting to same parent is ignored
        new_desc = "My new description"
        cmd = base_cmd + f"type set '{type_name4}' --parent '{type_name2}' --desc '{new_desc}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        # make sure description was updated, yet parent remains
        entries = self.get_cli_entries(cmd_env, base_cmd + "ty ls -f json", "parameter-type")
        entry = find_by_prop(entries, "Name", type_name4)[0]
        self.assertEqual(entry.get("Parent"), type_name2)
        self.assertEqual(entry.get("Description"), new_desc)

        # cleanup - unwind the stack
        self.delete_type(cmd_env, type_name4)
        self.delete_type(cmd_env, type_name3)
        self.delete_type(cmd_env, type_name2)
        self.delete_type(cmd_env, type_name1)

    def test_type_pagination(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        page_size = TEST_PAGE_SIZE
        type_count = page_size + 1

        for idx in range(type_count):
            type_name = self.make_name(f"test-pag-{idx}")
            self.create_type(cmd_env, type_name)

        self.assertPaginated(cmd_env, base_cmd + "type ls", "/types/?")

        # cleanup
        for idx in range(type_count):
            type_name = self.make_name(f"test-pag-{idx}")
            self.delete_type(cmd_env, type_name)

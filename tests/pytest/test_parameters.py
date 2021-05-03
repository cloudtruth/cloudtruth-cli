from testcase import TestCase


class TestParameters(TestCase):
    def test_parameter_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = "test-param-basic"
        empty_msg = f"No parameters found in project {proj_name}"
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # same result with the --values flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # same result with the --values and --secrets flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0), "Initial empty parameters"
        self.assertTrue(result.out_contains_value(empty_msg))

        ########
        # add first, non-secret parameter
        key1 = "my_param"
        value1 = "cRaZy value"
        desc1 = "this is just a test description"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --value \"{value1}\" --desc \"{desc1}\"")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-------------+---------+---------------------------------+
| Name     | Value       | Source  | Description                     |
+----------+-------------+---------+---------------------------------+
| my_param | cRaZy value | default | this is just a test description |
+----------+-------------+---------+---------------------------------+
""")

        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # delete the project
        self.delete_project(cmd_env, proj_name)

    def test_parameter_secret(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = "test-param-secret"
        empty_msg = f"No parameters found in project {proj_name}"
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # same result with the --values flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # same result with the --values and --secrets flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0), "Initial empty parameters"
        self.assertTrue(result.out_contains_value(empty_msg))

        ########
        # add first, secret parameter
        key1 = "my_param"
        value1 = "super-SENSITIVE-vAluE"
        desc1 = "my secret value"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --secret true --value \"{value1}\" --desc \"{desc1}\"")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-------+---------+-----------------+
| Name     | Value | Source  | Description     |
+----------+-------+---------+-----------------+
| my_param | ***** | default | my secret value |
+----------+-------+---------+-----------------+
""")

        # now, display with the secrets value
        result = self.run_cli(cmd_env, sub_cmd + f"list --values --secrets")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-----------------------+---------+-----------------+
| Name     | Value                 | Source  | Description     |
+----------+-----------------------+---------+-----------------+
| my_param | super-SENSITIVE-vAluE | default | my secret value |
+----------+-----------------------+---------+-----------------+
""")

        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # delete the project
        self.delete_project(cmd_env, proj_name)

    def test_project_separation(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name1 = "proj-sep1"
        proj_name2 = "proj-sep2"

        self.create_project(cmd_env, proj_name1)
        self.create_project(cmd_env, proj_name2)

        var1_name = "sna"
        var1_value1 = "foo"
        var1_value2 = "fu"
        var2_name = "sensitive"
        var2_value1 = "classified"
        var2_value2 = "top-secret"
        self.set_param(cmd_env, proj_name1, var1_name, var1_value1)
        self.set_param(cmd_env, proj_name1, var2_name, var2_value1, True)
        self.set_param(cmd_env, proj_name2, var1_name, var1_value2)
        self.set_param(cmd_env, proj_name2, var2_name, var2_value2, True)

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name1} param ls -v -s")
        self.assertEqual(result.out(), """\
+-----------+------------+---------+-------------+
| Name      | Value      | Source  | Description |
+-----------+------------+---------+-------------+
| sensitive | classified | default |             |
| sna       | foo        | default |             |
+-----------+------------+---------+-------------+
""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param ls -v -s")
        self.assertEqual(result.out(), """\
+-----------+------------+---------+-------------+
| Name      | Value      | Source  | Description |
+-----------+------------+---------+-------------+
| sensitive | top-secret | default |             |
| sna       | fu         | default |             |
+-----------+------------+---------+-------------+
""")

        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)

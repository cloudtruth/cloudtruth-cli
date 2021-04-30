from testcase import TestCase

class TestParameters(TestCase):
    def test_parameter_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = "test-param-basic"
        empty_msg = f"No parameters found in project {proj_name}"
        result = self.run_cli(cmd_env,base_cmd + f" projects set {proj_name} -d 'test_parameters_basic_crud() test'")
        self.assertEqual(result.return_value, 0)
    
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
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value \"{value1}\" --desc \"{desc1}\"")
        self.assertEqual(result.return_value, 0)
    
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertTrue(result.out_equals("""\
+----------+-------------+---------+---------------------------------+
| Name     | Value       | Source  | Description                     |
+----------+-------------+---------+---------------------------------+
| my_param | cRaZy value | default | this is just a test description |
+----------+-------------+---------+---------------------------------+
"""))

        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)
    
        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))
    
        # delete the project
        result = self.run_cli(cmd_env,base_cmd + f" projects delete {proj_name} --confirm")
        self.assertEqual(result.return_value, 0)


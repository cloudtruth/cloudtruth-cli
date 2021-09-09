import datetime
import os

from typing import Tuple, Dict
from testcase import TestCase, DEFAULT_ENV_NAME, REDACTED, DEFAULT_PARAM_VALUE
from testcase import PROP_CREATED, PROP_MODIFIED, PROP_VALUE


class TestParameters(TestCase):

    def _empty_message(self, proj_name: str) -> str:
        return f"No parameters found in project {proj_name}"

    def test_parameter_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-basic")
        empty_msg = self._empty_message(proj_name)
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
                              sub_cmd + f"set {key1} --value '{value1}' --desc '{desc1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type   | Secret | Description                     |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | static | false  | this is just a test description |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
my_param,cRaZy value,default,string,0,static,false,this is just a test description
""")
        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{value1}", result.out())

        # get the parameter details
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1} --details")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Name: {key1}", result.out())
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn("Source: default", result.out())
        self.assertIn("Secret: false", result.out())
        self.assertIn(f"Description: {desc1}", result.out())

        # idempotent -- same value
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value1, result.out())

        ########
        # update the just the value
        value2 = "new_value"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # rename the parameter
        orig_name = key1
        key1 = "renamed_param"
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} -r {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully updated parameter '{key1}'", result.out())

        ########
        # update the just the description
        desc2 = "alt description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} -d '{desc2}'")
        self.assertEqual(result.return_value, 0)

        ########
        # no updates provided
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Please provide at least one update", result.err())

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc2))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete --yes '{key1}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully removed parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # idempotent
        result = self.run_cli(cmd_env, sub_cmd + f"delete -y '{key1}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Did not find parameter '{key1}'", result.out())

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
        proj_name = self.make_name("test-param-secret")
        empty_msg = self._empty_message(proj_name)
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
                              sub_cmd + f"set {key1} --secret true --value '{value1}' --desc '{desc1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-------+---------+------------+-------+--------+--------+-----------------+
| Name     | Value | Source  | Param Type | Rules | Type   | Secret | Description     |
+----------+-------+---------+------------+-------+--------+--------+-----------------+
| my_param | ***** | default | string     | 0     | static | true   | my secret value |
+----------+-------+---------+------------+-------+--------+--------+-----------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), f"""\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
my_param,{REDACTED},default,string,0,static,true,my secret value
""")

        # now, display with the secrets value
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-----------------------+---------+------------+-------+--------+--------+-----------------+
| Name     | Value                 | Source  | Param Type | Rules | Type   | Secret | Description     |
+----------+-----------------------+---------+------------+-------+--------+--------+-----------------+
| my_param | super-SENSITIVE-vAluE | default | string     | 0     | static | true   | my secret value |
+----------+-----------------------+---------+------------+-------+--------+--------+-----------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets --format csv")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
my_param,super-SENSITIVE-vAluE,default,string,0,static,true,my secret value
""")

        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{value1}", result.out())

        # get the parameter details
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1} --details")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Name: {key1}", result.out())
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn("Source: default", result.out())
        self.assertIn("Secret: true", result.out())
        self.assertIn(f"Description: {desc1}", result.out())

        # idempotent -- same value
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))

        # make sure it is still a secret
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value1, result.out())

        ########
        # update the just the value
        value2 = "new_value"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # update the just the description
        desc2 = "alt description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} -d '{desc2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc2))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete -y '{key1}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully removed parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # idempotent
        result = self.run_cli(cmd_env, sub_cmd + f"delete -y '{key1}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Did not find parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # delete the project
        self.delete_project(cmd_env, proj_name)

    def test_parameter_project_separation(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name1 = self.make_name("proj-sep1")
        proj_name2 = self.make_name("proj-sep2")

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
+-----------+------------+---------+------------+-------+--------+--------+-------------+
| Name      | Value      | Source  | Param Type | Rules | Type   | Secret | Description |
+-----------+------------+---------+------------+-------+--------+--------+-------------+
| sensitive | classified | default | string     | 0     | static | true   |             |
| sna       | foo        | default | string     | 0     | static | false  |             |
+-----------+------------+---------+------------+-------+--------+--------+-------------+
""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param ls -v -s")
        self.assertEqual(result.out(), """\
+-----------+------------+---------+------------+-------+--------+--------+-------------+
| Name      | Value      | Source  | Param Type | Rules | Type   | Secret | Description |
+-----------+------------+---------+------------+-------+--------+--------+-------------+
| sensitive | top-secret | default | string     | 0     | static | true   |             |
| sna       | fu         | default | string     | 0     | static | false  |             |
+-----------+------------+---------+------------+-------+--------+--------+-------------+
""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name1} param export docker -s")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
SENSITIVE=classified
SNA=foo

""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param export docker -s")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
SENSITIVE=top-secret
SNA=fu

""")

        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)

    def test_parameter_environment_separation(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name = self.make_name("baseball")
        self.create_project(cmd_env, proj_name)

        env_name1 = DEFAULT_ENV_NAME  # no job-id variation
        env_name2 = self.make_name("test-mets")
        self.create_environment(cmd_env, env_name2)
        env_name3 = self.make_name("test-redsox")
        self.create_environment(cmd_env, env_name3, parent=env_name2)

        var1_name = "base"
        var1_value1 = "first"
        var1_value2 = "second"
        var1_value3 = "third"
        var2_name = "pitch"
        var2_value1 = "slider"
        var2_value2 = "split"
        var2_value3 = "heater"
        self.set_param(cmd_env, proj_name, var1_name, var1_value1)
        self.set_param(cmd_env, proj_name, var2_name, var2_value1)

        proj_cmd = base_cmd + f"--project '{proj_name}' "
        # NOTE: due to environment name in Source column, cannot do an absolute string
        param_ls = " param ls -v -s -f csv"
        env1_list = proj_cmd + param_ls
        env2_list = proj_cmd + f"--env '{env_name2}'" + param_ls
        env3_list = proj_cmd + f"--env '{env_name3}'" + param_ls

        # see that values are inherited in the different environments
        result = self.run_cli(cmd_env, env1_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, proj_cmd + "param env 'no-such-parameter' -f csv")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Parameter 'no-such-parameter' was not found", result.err())

        result = self.run_cli(cmd_env, proj_cmd + f"param env {var1_name} -f csv")
        self.assertIn(f"{env_name1},{var1_value1},,", result.out())
        self.assertNotIn(env_name2, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"param env {var1_name} -f csv --all")
        self.assertIn(f"{env_name1},{var1_value1},,", result.out())
        self.assertIn(f"{env_name2},-,,", result.out())

        # add the parameters for the second environment
        self.set_param(cmd_env, proj_name, var1_name, var1_value2, env=env_name2)
        self.set_param(cmd_env, proj_name, var2_name, var2_value2, True, env=env_name2)

        # see that values are inherited in the different environments
        result = self.run_cli(cmd_env, env1_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertIn(f"{var1_name},{var1_value2},{env_name2}", result.out())
        self.assertIn(f"{var2_name},{var2_value2},{env_name2}", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertIn(f"{var1_name},{var1_value2},{env_name2}", result.out())
        self.assertIn(f"{var2_name},{var2_value2},{env_name2}", result.out())

        # add the parameters for the third environment
        self.set_param(cmd_env, proj_name, var1_name, var1_value3, env=env_name3)
        self.set_param(cmd_env, proj_name, var2_name, var2_value3, True, env=env_name3)

        # see that secrets do not show up without -s
        result = self.run_cli(cmd_env, proj_cmd + f"param environment '{var2_name}' -f csv")
        self.assertIn(f"{env_name1},{REDACTED},,", result.out())
        self.assertIn(f"{env_name2},{REDACTED},,", result.out())
        self.assertIn(f"{env_name3},{REDACTED},,", result.out())

        # see that secrets do not show up without -s
        result = self.run_cli(cmd_env, proj_cmd + f"param environment '{var2_name}' -f csv -s")
        self.assertIn(f"{env_name1},{var2_value1},,", result.out())
        self.assertIn(f"{env_name2},{var2_value2},,", result.out())
        self.assertIn(f"{env_name3},{var2_value3},,", result.out())

        # see that values are inherited in the different environments
        result = self.run_cli(cmd_env, env1_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertIn(f"{var1_name},{var1_value2},{env_name2}", result.out())
        self.assertIn(f"{var2_name},{var2_value2},{env_name2}", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertIn(f"{var1_name},{var1_value3},{env_name3}", result.out())
        self.assertIn(f"{var2_name},{var2_value3},{env_name3}", result.out())

        docker_cmd = " param export docker -s"
        result = self.run_cli(cmd_env, proj_cmd + docker_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
{var1_name.upper()}={var1_value1}
{var2_name.upper()}={var2_value1}

""")

        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name2}" + docker_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
{var1_name.upper()}={var1_value2}
{var2_name.upper()}={var2_value2}

""")

        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name3}" + docker_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
{var1_name.upper()}={var1_value3}
{var2_name.upper()}={var2_value3}

""")

        # remove env2 override
        unset_cmd = f"param unset '{var1_name}'"
        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name2} " + unset_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully removed parameter value '{var1_name}'", result.out())
        self.assertIn(f"for environment '{env_name2}'", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertIn(f"{var1_name},{var1_value3},{env_name3}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env1_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        # remove env3 override
        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name3} " + unset_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully removed parameter value '{var1_name}'", result.out())
        self.assertIn(f"for environment '{env_name3}'", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env1_list)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        # cleanup -- environments must be in reverse order
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name3)
        self.delete_environment(cmd_env, env_name2)

    def test_parameter_export(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-export")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters -- to avoid later confusion
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        ########
        # add first, non-secret parameter
        key1 = "first_param"
        value1 = "posix_compliant_value"
        self.set_param(cmd_env, proj_name, key1, value1)

        # add first, non-secret parameter
        key2 = "SECOND_PARAM"
        value2 = "a value with spaces"
        self.set_param(cmd_env, proj_name, key2, value2)

        # add a non-posix complaint key with a posix value
        key3 = "non.posix.key"
        value3 = "posix_value_invalid_key"
        self.set_param(cmd_env, proj_name, key3, value3)

        # add first, secret parameter
        key4 = "FIRST_PARAM_SECRET"
        value4 = "top-secret-sci"
        self.set_param(cmd_env, proj_name, key4, value4, secret=True)

        # add first, secret parameter
        key5 = "second_secret"
        value5 = "sensitive value with spaces"
        self.set_param(cmd_env, proj_name, key5, value5, secret=True)

        #####################
        # Docker
        docker_cmd = base_cmd + f"--project {proj_name} param export docker "
        result = self.run_cli(cmd_env, docker_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET={REDACTED}
SECOND_PARAM=a value with spaces
SECOND_SECRET={REDACTED}

""")

        result = self.run_cli(cmd_env, docker_cmd + "--secrets")
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces
SECOND_SECRET=sensitive value with spaces

""")

        result = self.run_cli(cmd_env, docker_cmd + "--secrets --starts-with SECOND")
        self.assertEqual(result.out(), """\
SECOND_PARAM=a value with spaces
SECOND_SECRET=sensitive value with spaces

""")

        # use uppercase key without secrets
        result = self.run_cli(cmd_env, docker_cmd + "--starts-with FIRST")
        self.assertEqual(result.out(), f"""\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET={REDACTED}

""")

        # use uppercase key with secrets
        result = self.run_cli(cmd_env, docker_cmd + "--starts-with FIRST -s")
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci

""")

        # use lowercase key with secrets
        result = self.run_cli(cmd_env, docker_cmd + "--contains param -s")
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces

""")

        # see if filter picks up non-posix
        result = self.run_cli(cmd_env, docker_cmd + "--contains posix -s")
        self.assertEqual(result.out(), """\

""")

        #####################
        # Dotenv
        dotenv_cmd = base_cmd + f"--project {proj_name} param export dotenv "
        result = self.run_cli(cmd_env, dotenv_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM="posix_compliant_value"
FIRST_PARAM_SECRET="{REDACTED}"
SECOND_PARAM="a value with spaces"
SECOND_SECRET="{REDACTED}"

""")

        dotenv_cmd = base_cmd + f"--project {proj_name} param export dotenv -s"
        result = self.run_cli(cmd_env, dotenv_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
FIRST_PARAM="posix_compliant_value"
FIRST_PARAM_SECRET="top-secret-sci"
SECOND_PARAM="a value with spaces"
SECOND_SECRET="sensitive value with spaces"

""")
        #####################
        # Shell
        shell_cmd = base_cmd + f"--project {proj_name} param export shell "
        result = self.run_cli(cmd_env, shell_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET={REDACTED}
SECOND_PARAM='a value with spaces'
SECOND_SECRET={REDACTED}

""")

        shell_cmd = base_cmd + f"--project {proj_name} param export shell -s"
        result = self.run_cli(cmd_env, shell_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM='a value with spaces'
SECOND_SECRET='sensitive value with spaces'

""")

        # cleanup (no need to delete individual parameters)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_secret_switch(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-secret-switch")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0), "Initial empty parameters"
        self.assertTrue(result.out_contains_value(empty_msg))

        ########
        # add first, non-secret parameter
        key1 = "my_param"
        value1 = "cRaZy value"
        desc1 = "this is just a test description"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --value '{value1}' --desc '{desc1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type   | Secret | Description                     |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | static | false  | this is just a test description |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
""")

        # switch it to a secret
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret true")
        self.assertEqual(result.return_value, 0)

        # see that it has been changed to a secret (redacted in cli)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------+---------+------------+-------+--------+--------+---------------------------------+
| Name     | Value | Source  | Param Type | Rules | Type   | Secret | Description                     |
+----------+-------+---------+------------+-------+--------+--------+---------------------------------+
| my_param | ***** | default | string     | 0     | static | true   | this is just a test description |
+----------+-------+---------+------------+-------+--------+--------+---------------------------------+
""")

        # verify value has not changed
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type   | Secret | Description                     |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | static | true   | this is just a test description |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
""")

        # switch back to a regular parameter
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret false")
        self.assertEqual(result.return_value, 0)

        # see that it is no longer redacted
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type   | Secret | Description                     |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | static | false  | this is just a test description |
+----------+-------------+---------+------------+-------+--------+--------+---------------------------------+
""")

        self.delete_project(cmd_env, proj_name)

    def test_parameter_local_file(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # create the file with the value
        filename = self.make_name("value")
        value1 = "static val from file"
        file = open(filename, "w")
        file.write(value1)
        file.close()

        # add a new project
        proj_name = self.make_name("test-local-file")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0), "Initial empty parameters"
        self.assertTrue(result.out_contains_value(empty_msg))

        ########
        # add first, non-secret parameter from file
        key1 = "my_param"
        desc1 = "param set from file input"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --input '{filename}' --desc '{desc1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+----------------------+---------+------------+-------+--------+--------+---------------------------+
| Name     | Value                | Source  | Param Type | Rules | Type   | Secret | Description               |
+----------+----------------------+---------+------------+-------+--------+--------+---------------------------+
| my_param | static val from file | default | string     | 0     | static | false  | param set from file input |
+----------+----------------------+---------+------------+-------+--------+--------+---------------------------+
""")

        # change value from `--value` flag from CLI
        value2 = "update-from-value"
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --value '{value2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------------+---------+------------+-------+--------+--------+---------------------------+
| Name     | Value             | Source  | Param Type | Rules | Type   | Secret | Description               |
+----------+-------------------+---------+------------+-------+--------+--------+---------------------------+
| my_param | update-from-value | default | string     | 0     | static | false  | param set from file input |
+----------+-------------------+---------+------------+-------+--------+--------+---------------------------+
""")

        # update with a different value from file
        value3 = "another-static-file"
        file = open(filename, "w")
        file.write(value3)
        file.close()

        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --input '{filename}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+---------------------+---------+------------+-------+--------+--------+---------------------------+
| Name     | Value               | Source  | Param Type | Rules | Type   | Secret | Description               |
+----------+---------------------+---------+------------+-------+--------+--------+---------------------------+
| my_param | another-static-file | default | string     | 0     | static | false  | param set from file input |
+----------+---------------------+---------+------------+-------+--------+--------+---------------------------+
""")

        # cleanup
        os.remove(filename)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_integration_errors(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-int-err")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0), "Initial empty parameters"
        self.assertTrue(result.out_contains_value(empty_msg))

        key1 = "param1"
        value1 = "value"
        fqn = "GitHub::bogus::repo::directory::file"
        jmes = "foo.bar"
        conflict_msg = "Conflicting arguments: cannot specify"
        invalid_fqn_msg = "Invalid FQN or JMES path expression"

        #####################
        # verify over specifying
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' -v '{value1}' --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(conflict_msg, result.err())

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --prompt --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(conflict_msg, result.err())

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --input 'missing.txt' --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(conflict_msg, result.err())

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --prompt --jmes '{jmes}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(conflict_msg, result.err())

        # check that nothing was added
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertTrue(result.out_contains_value(empty_msg))

        #####################
        # poorly structured FQN
        completely_bogus_msg = "missing the network location"

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())
        self.assertIn(completely_bogus_msg, result.err())

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())
        self.assertIn(completely_bogus_msg, result.err())

        #####################
        # no such FQN provider
        fqn = "foobar://bogus::repo::directory::file"
        no_provider_msg = "No integration provider available for"

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())
        self.assertIn(no_provider_msg, result.err())

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())
        self.assertIn(no_provider_msg, result.err())

        #####################
        # no such FQN, but a legit provider
        fqn = "github://this-is-a-crazy/repo-path/that/does/not/exist"
        no_integration_msg = "No integration available for"

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())
        self.assertIn(no_integration_msg, result.err())

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())
        self.assertIn(no_integration_msg, result.err())

        # check that nothing was added
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets -f csv")
        self.assertTrue(result.out_contains_value(empty_msg))

        # verify `--dynamic` flag causes specialized warning
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        empty_msg = f"No dynamic parameters found in project {proj_name}"
        result = self.run_cli(cmd_env, sub_cmd + "list --dynamic")
        self.assertTrue(result.out_contains_value(empty_msg))

        result = self.run_cli(cmd_env, sub_cmd + "list --dynamic -v")
        self.assertTrue(result.out_contains_value(empty_msg))

        result = self.run_cli(cmd_env, sub_cmd + "list --dynamic -v -s")
        self.assertTrue(result.out_contains_value(empty_msg))

        result = self.run_cli(cmd_env, sub_cmd + "list --dynamic -v -s --show-times")
        self.assertTrue(result.out_contains_value(empty_msg))

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_parameter_table_formats(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-tables")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        ########
        # add a couple parameters
        key1 = "speicla3"
        value1 = "beef brocolli, pork fried rice"
        desc1 = "Jade lunch"

        key2 = "speicla14"
        value2 = "cueey-chicken"
        desc2 = "Jade secret"

        self.set_param(cmd_env, proj_name, key1, value1, desc=desc1)
        self.set_param(cmd_env, proj_name, key2, value2, secret=True, desc=desc2)

        #################
        # table format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+-----------+--------------------------------+---------+------------+-------+--------+--------+-------------+
| Name      | Value                          | Source  | Param Type | Rules | Type   | Secret | Description |
+-----------+--------------------------------+---------+------------+-------+--------+--------+-------------+
| speicla14 | *****                          | default | string     | 0     | static | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | string     | 0     | static | false  | Jade lunch  |
+-----------+--------------------------------+---------+------------+-------+--------+--------+-------------+
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertEqual(result.out(), """\
+-----------+--------------------------------+---------+------------+-------+--------+--------+-------------+
| Name      | Value                          | Source  | Param Type | Rules | Type   | Secret | Description |
+-----------+--------------------------------+---------+------------+-------+--------+--------+-------------+
| speicla14 | cueey-chicken                  | default | string     | 0     | static | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | string     | 0     | static | false  | Jade lunch  |
+-----------+--------------------------------+---------+------------+-------+--------+--------+-------------+
""")

        #################
        # CSV format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertEqual(result.out(), f"""\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
speicla14,{REDACTED},default,string,0,static,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,string,0,static,false,Jade lunch
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv -s")
        self.assertEqual(result.out(), """\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
speicla14,cueey-chicken,default,string,0,static,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,string,0,static,false,Jade lunch
""")

        #################
        # JSON format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f json")
        self.assertEqual(result.out(), """\
{
  "parameter": [
    {
      "Description": "Jade secret",
      "Name": "speicla14",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "true",
      "Source": "default",
      "Type": "static",
      "Value": "*****"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "static",
      "Value": "beef brocolli, pork fried rice"
    }
  ]
}
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f json -s")
        self.assertEqual(result.out(), """\
{
  "parameter": [
    {
      "Description": "Jade secret",
      "Name": "speicla14",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "true",
      "Source": "default",
      "Type": "static",
      "Value": "cueey-chicken"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "static",
      "Value": "beef brocolli, pork fried rice"
    }
  ]
}
""")

        #################
        # YAML format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f yaml")
        self.assertEqual(result.out(), f"""\
---
parameter:
  - Description: Jade secret
    Name: speicla14
    Param Type: string
    Rules: "0"
    Secret: "true"
    Source: default
    Type: static
    Value: "{REDACTED}"
  - Description: Jade lunch
    Name: speicla3
    Param Type: string
    Rules: "0"
    Secret: "false"
    Source: default
    Type: static
    Value: "beef brocolli, pork fried rice"
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f yaml -s")
        self.assertEqual(result.out(), """\
---
parameter:
  - Description: Jade secret
    Name: speicla14
    Param Type: string
    Rules: "0"
    Secret: "true"
    Source: default
    Type: static
    Value: cueey-chicken
  - Description: Jade lunch
    Name: speicla3
    Param Type: string
    Rules: "0"
    Secret: "false"
    Source: default
    Type: static
    Value: "beef brocolli, pork fried rice"
""")

        # delete the project
        self.delete_project(cmd_env, proj_name)

    def test_parameter_names(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-names")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f"--project '{proj_name}' param "
        show_cmd = sub_cmd + "list -vsf csv"
        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        param_value = "something"
        names = [
            "simple_underscore",
            "simple.dot",
            "simple/slash",
            "simple space",
            "MixCase",
        ]
        for param_name in names:
            # create the initial parameter
            self.set_param(cmd_env, proj_name, param_name, param_value)
            self.verify_param(cmd_env, proj_name, param_name, param_value)

            # rename it
            temp_name = "foo"
            self.run_cli(cmd_env, sub_cmd + f"set -r '{temp_name}' '{param_name}'")
            self.verify_param(cmd_env, proj_name, temp_name, param_value)

            # back to the original name
            self.run_cli(cmd_env, sub_cmd + f"set -r '{param_name}' '{temp_name}'")
            self.verify_param(cmd_env, proj_name, param_name, param_value)

            self.delete_param(cmd_env, proj_name, param_name)

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_parameter_diff(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-cmp")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # add a couple environments
        env_a = self.make_name("left")
        self.create_environment(cmd_env, env_a)
        env_b = self.make_name("right")
        self.create_environment(cmd_env, env_b)

        # check that there are no parameters
        sub_cmd = base_cmd + f"--project '{proj_name}' param "
        show_cmd = sub_cmd + "list -vsf csv"
        result = self.run_cli(cmd_env, show_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(empty_msg, result.out())

        param1 = "param1"
        param2 = "secret1"

        # add some parameters to ENV A
        value1a = "some_value"
        value2a = "ssshhhh"
        self.set_param(cmd_env, proj_name, param1, value1a, env=env_a)
        self.set_param(cmd_env, proj_name, param2, value2a, env=env_a, secret=True)

        # first set of comparisons
        diff_cmd = sub_cmd + f"diff -e '{env_a}' --env '{env_b}' -f csv "
        result = self.run_cli(cmd_env, diff_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param1},{value1a},{DEFAULT_PARAM_VALUE}
{param2},{REDACTED},{DEFAULT_PARAM_VALUE}
""")

        result = self.run_cli(cmd_env, diff_cmd + "-s")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param1},{value1a},{DEFAULT_PARAM_VALUE}
{param2},{value2a},{DEFAULT_PARAM_VALUE}
""")

        # set some stuff in the default environment
        value1d = "different"
        value2d = "be qwiet"
        self.set_param(cmd_env, proj_name, param1, value1d)
        self.set_param(cmd_env, proj_name, param2, value2d)

        # values from the default environment should show up
        result = self.run_cli(cmd_env, diff_cmd + "-s")
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param1},{value1a},{value1d}
{param2},{value2a},{value2d}
""")

        # now, let's see the properties
        result = self.run_cli(cmd_env, diff_cmd + "-s -p value -p environment ")
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param1},"{value1a},\n{env_a}","{value1d},\ndefault"
{param2},"{value2a},\n{env_a}","{value2d},\ndefault"
""")

        # now, set some different values
        same = "matchers"
        value2b = "im hunting wabbits"
        self.set_param(cmd_env, proj_name, param1, same, env=env_a)
        self.set_param(cmd_env, proj_name, param1, same, env=env_b)
        self.set_param(cmd_env, proj_name, param2, value2b, env=env_b)

        # without the --all flag, only the deltas are shown
        result = self.run_cli(cmd_env, diff_cmd + "-s")
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param2},{value2a},{value2b}
""")

        def split_time_strings(value: str) -> Tuple:
            # the values includes an extra '\n' for improved display in the table, and the Python
            # parser does not like the trailing 'Z'.
            return value.replace("\n", "").replace("\'", "").replace("Z", "").split(",")

        # check that we get back timestamp properties
        diff_json_cmd = sub_cmd + f"diff -e '{env_a}' -e '{env_b}' -f json "
        result = self.run_cli(cmd_env, diff_json_cmd + "-p created-at --property modified-at")
        self.assertEqual(result.return_value, 0)
        output = eval(result.out())

        p1_entry = output["parameter"][0]
        self.assertEqual(p1_entry["Parameter"], param1)
        (created_a, modified_a) = split_time_strings(p1_entry[env_a])
        self.assertIsNotNone(datetime.datetime.fromisoformat(created_a))
        self.assertIsNotNone(datetime.datetime.fromisoformat(modified_a))
        (created_b, modified_b) = split_time_strings(p1_entry[env_b])
        self.assertIsNotNone(datetime.datetime.fromisoformat(created_b))
        self.assertIsNotNone(datetime.datetime.fromisoformat(modified_b))

        p2_entry = output["parameter"][1]
        self.assertEqual(p2_entry["Parameter"], param2)
        (created_a, modified_a) = split_time_strings(p2_entry[env_a])
        self.assertIsNotNone(datetime.datetime.fromisoformat(created_a))
        self.assertIsNotNone(datetime.datetime.fromisoformat(modified_a))
        (created_b, modified_b) = split_time_strings(p2_entry[env_b])
        self.assertIsNotNone(datetime.datetime.fromisoformat(created_b))
        self.assertIsNotNone(datetime.datetime.fromisoformat(modified_b))

        # when specifying properties where there are no diffs, we get nothing
        result = self.run_cli(cmd_env, diff_cmd + "-s --property fqn")
        self.assertIn("No parameters or differences in compared properties found", result.out())

        #####################
        # Time diff
        diff_csv = sub_cmd + "diff -f csv "

        # test single time
        result = self.run_cli(cmd_env, diff_csv + f"--as-of '{modified_a}'")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
Parameter,Current,{modified_a}
{param1},{value1d},-
{param2},{REDACTED},-
""")

        # compare 2 points in time (with secrets)
        result = self.run_cli(cmd_env, diff_csv + f"-s --as-of '{modified_a}' --as-of  '{modified_b}'")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
Parameter,{modified_a},{modified_b}
{param1},-,{value1d}
{param2},-,{value2d}
""")

        # compare 2 points in time where there are no differences
        result = self.run_cli(cmd_env, diff_csv + f"--as-of '{created_a}' --as-of '{modified_a}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn("No parameters or differences in compared properties found", result.out())

        #####################
        # Combination environment/time diff

        # if just one env/time, it applies to the right hand side
        result = self.run_cli(cmd_env, diff_csv + f"-e '{env_a}' --as-of '{modified_a}'")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
Parameter,default,{env_a} ({modified_a})
{param1},{value1d},{value1a}
""")

        # the full set of environments/times (with secrets)
        cmd = diff_csv + f"-e '{env_a}' --as-of '{modified_a}' -e '{env_b}' --as-of '{modified_b}' -s"
        result = self.run_cli(cmd_env, cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), f"""\
Parameter,{env_a} ({modified_a}),{env_b} ({modified_b})
{param1},{value1a},{same}
{param2},{value2a},{value2b}
""")

        #####################
        # Error cases

        # no comparing to yourself
        result = self.run_cli(cmd_env, sub_cmd + "difference")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Invalid comparing an environment to itself", result.err())

        matched_envs = f"-e '{env_a}' " * 2
        result = self.run_cli(cmd_env, sub_cmd + f"difference {matched_envs}")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Invalid comparing an environment to itself", result.err())

        matched_times = "--as-of 2021-08-27 " * 2
        result = self.run_cli(cmd_env, sub_cmd + f"difference {matched_times}")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Invalid comparing an environment to itself", result.err())

        result = self.run_cli(cmd_env, sub_cmd + f"difference {matched_times} {matched_envs}")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Invalid comparing an environment to itself", result.err())

        # first environment DNE
        result = self.run_cli(cmd_env, sub_cmd + "differ -e 'charlie-foxtrot' -e '{env_b}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Did not find environment 'charlie-foxtrot'", result.err())

        # second environment DNE
        result = self.run_cli(cmd_env, sub_cmd + f"differences -e '{env_a}' -e 'missing'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Did not find environment 'missing'", result.err())

        # too many specified
        result = self.run_cli(cmd_env, sub_cmd + "diff -e env1 --env env2 -e env3")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Can specify a maximum of 2 environment values", result.err())

        result = self.run_cli(cmd_env, sub_cmd + "diff --as-of 2021-08-01 --as-of 2021-08-02 --as-of 2021-08-03")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Can specify a maximum of 2 as-of values", result.err())

        # cleanup
        self.delete_environment(cmd_env, env_a)
        self.delete_environment(cmd_env, env_b)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_times(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-times")
        self.create_project(cmd_env, proj_name)

        # add a couple environments
        env_a = self.make_name("env-a-time")
        self.create_environment(cmd_env, env_a)
        env_b = self.make_name("env-b-time")
        self.create_environment(cmd_env, env_b)

        param1 = "some_param"
        value_a1 = "value a - first"
        value_a2 = "value a - second"
        value_b1 = "value B1"
        value_b2 = "value B2"
        self.set_param(cmd_env, proj_name, param1, value_a1, env=env_a)
        self.set_param(cmd_env, proj_name, param1, value_b1, env=env_b)

        # fetch complete details for first set
        details_a1 = self.get_param(cmd_env, proj_name, param1, env=env_a)
        self.assertIsNotNone(details_a1)
        self.assertEqual(details_a1.get(PROP_VALUE), value_a1)
        details_b1 = self.get_param(cmd_env, proj_name, param1, env=env_b)
        self.assertIsNotNone(details_b1)
        self.assertEqual(details_b1.get(PROP_VALUE), value_b1)

        # get the newest time from the first set of changes
        modified_at = details_b1.get(PROP_MODIFIED)

        # update values
        self.set_param(cmd_env, proj_name, param1, value_a2, env=env_a)
        self.set_param(cmd_env, proj_name, param1, value_b2, env=env_b)

        # sanity checks on updated values
        details_a2 = self.get_param(cmd_env, proj_name, param1, env=env_a)
        self.assertIsNotNone(details_a2)
        self.assertEqual(details_a2.get(PROP_VALUE), value_a2)
        details_b2 = self.get_param(cmd_env, proj_name, param1, env=env_b)
        self.assertIsNotNone(details_b2)
        self.assertEqual(details_b2.get(PROP_VALUE), value_b2)

        ####################
        # verify the 'get' command returns the correct values
        # NOTE: this leverages the verify_param(), since it uses the 'param get' command
        self.verify_param(cmd_env, proj_name, param1, value_a1, env=env_a, time=details_a1.get(PROP_MODIFIED))
        self.verify_param(cmd_env, proj_name, param1, value_b1, env=env_b, time=details_b1.get(PROP_MODIFIED))

        self.verify_param(cmd_env, proj_name, param1, value_a2, env=env_a, time=details_a2.get(PROP_MODIFIED))
        self.verify_param(cmd_env, proj_name, param1, value_b2, env=env_b, time=details_b2.get(PROP_MODIFIED))

        # check with details turned on
        cmd = base_cmd + f"--project '{proj_name}' --env '{env_a}' param get '{param1}' -d"
        result = self.run_cli(cmd_env, cmd)
        self.assertIn(f"{PROP_CREATED}: {details_a2.get(PROP_CREATED)}", result.out())
        self.assertIn(f"{PROP_MODIFIED}: {details_a2.get(PROP_MODIFIED)}", result.out())
        self.assertIn(f"{PROP_VALUE}: {details_a2.get(PROP_VALUE)}", result.out())

        # time filtered with details with other environment
        cmd = base_cmd + f"--project '{proj_name}' --env '{env_b}' param get '{param1}' -d --as-of {modified_at}"
        result = self.run_cli(cmd_env, cmd)
        self.assertIn(f"{PROP_CREATED}: {details_b1.get(PROP_CREATED)}", result.out())
        self.assertIn(f"{PROP_MODIFIED}: {details_b1.get(PROP_MODIFIED)}", result.out())
        self.assertIn(f"{PROP_VALUE}: {details_b1.get(PROP_VALUE)}", result.out())

        ####################
        # verify the 'list' command returns the correct values
        # NOTE: this leverages the get_param(), since it uses the 'param list' command
        self.assertEqual(details_a2, self.get_param(cmd_env, proj_name, param1, env=env_a))
        self.assertEqual(details_b2, self.get_param(cmd_env, proj_name, param1, env=env_b))

        self.assertEqual(details_a1, self.get_param(cmd_env, proj_name, param1, env=env_a, time=modified_at))
        self.assertEqual(details_b1, self.get_param(cmd_env, proj_name, param1, env=env_b, time=modified_at))

        ####################
        # verify the 'environments' command returns the correct values
        def equal_properties(entry: Dict, details: Dict) -> bool:
            return (entry.get(PROP_VALUE) == details.get(PROP_VALUE)
                    and entry.get(PROP_CREATED) == details.get(PROP_CREATED)
                    and entry.get(PROP_MODIFIED) == details.get(PROP_MODIFIED))

        param_cmd = base_cmd + f"--project '{proj_name}' param "
        env_cmd = param_cmd + f"env '{param1}' --show-times --format json "
        result = self.run_cli(cmd_env, env_cmd)
        data = eval(result.out())
        for item in data["parameter"]:
            if item.get("Environment") == env_a:
                self.assertTrue(equal_properties(item, details_a2))
            if item.get("Environment") == env_b:
                self.assertTrue(equal_properties(item, details_b2))

        env_cmd += f"--as-of {modified_at}"
        result = self.run_cli(cmd_env, env_cmd)
        data = eval(result.out())
        for item in data["parameter"]:
            if item.get("Environment") == env_a:
                self.assertTrue(equal_properties(item, details_a1))
            if item.get("Environment") == env_b:
                self.assertTrue(equal_properties(item, details_b1))

        ####################
        # parameter export now supports specifying --as-of
        export_cmd = base_cmd + f"--project '{proj_name}' --env '{env_a}' param export docker"
        result = self.run_cli(cmd_env, export_cmd)
        self.assertIn(f"{param1.upper()}={value_a2}", result.out())

        result = self.run_cli(cmd_env, export_cmd + f" --as-of {modified_at}")
        self.assertIn(f"{param1.upper()}={value_a1}", result.out())

        # cleanup
        self.delete_environment(cmd_env, env_a)
        self.delete_environment(cmd_env, env_b)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_types_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-basic-types")
        self.create_project(cmd_env, proj_name)
        param_cmd = base_cmd + f"--project {proj_name} param "
        list_cmd = param_cmd + "ls -v -f csv"
        type_err_msg = "Rule violation"

        #####################
        # boolean tests
        bool_param = "param1"
        bool_value = "true"
        bool_type = "bool"

        result = self.run_cli(cmd_env, param_cmd + f"set {bool_param} -t {bool_type} -v {bool_value}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully updated parameter '{bool_param}'", result.out())

        # see it in the display
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{bool_param},{bool_value},default,{bool_type},0,static,false", result.out())

        # try to set value to non-bool value
        result = self.run_cli(cmd_env, param_cmd + f"set {bool_param} -v not-a-bool")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(type_err_msg, result.err())
        self.assertIn("Value is not of type bool", result.err())

        # change the type back to string
        self.set_param(cmd_env, proj_name, bool_param, bool_value, param_type="string")
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{bool_param},{bool_value},default,string,0,static,false", result.out())

        # update back to bool
        self.set_param(cmd_env, proj_name, bool_param, bool_value, param_type=bool_type)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{bool_param},{bool_value},default,{bool_type},0,static,false", result.out())

        # toggle to secret
        self.set_param(cmd_env, proj_name, bool_param, bool_value, secret=True)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{bool_param},{REDACTED},default,{bool_type},0,static,true", result.out())

        # toggle back from secret
        self.set_param(cmd_env, proj_name, bool_param, bool_value, secret=False)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{bool_param},{bool_value},default,{bool_type},0,static,false", result.out())

        #####################
        # integer tests
        int_param = "param2"
        int_value = "-1234"
        int_type = "integer"

        result = self.run_cli(cmd_env, param_cmd + f"set {int_param} -t {int_type} -v {int_value}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Successfully updated parameter '{int_param}'", result.out())

        # see it in the display
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{int_param},{int_value},default,{int_type},0,static,false", result.out())

        # try to set value to non-integer value
        result = self.run_cli(cmd_env, param_cmd + f"set {int_param} -v not-an-integer")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(type_err_msg, result.err())
        self.assertIn("Value is not of type integer", result.err())

        # change the type back to string
        self.set_param(cmd_env, proj_name, int_param, int_value, param_type="string")
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{int_param},{int_value},default,string,0,static,false", result.out())

        # update back to integer
        self.set_param(cmd_env, proj_name, int_param, int_value, param_type=int_type)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{int_param},{int_value},default,{int_type},0,static,false", result.out())

        # toggle to secret
        self.set_param(cmd_env, proj_name, int_param, int_value, secret=True)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{int_param},{REDACTED},default,{int_type},0,static,true", result.out())

        # toggle back from secret
        self.set_param(cmd_env, proj_name, int_param, int_value, secret=False)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{int_param},{int_value},default,{int_type},0,static,false", result.out())

        # NOTE: no real need to test 'string' types, since that is the default and no illegal values

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_parameter_rules_strings(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-string-rules")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("string-env")
        self.create_environment(cmd_env, env_name)
        param_cmd = base_cmd + f"--project {proj_name} --env {env_name} param "
        list_cmd = param_cmd + "ls -v -f csv"
        rules_cmd = param_cmd + "ls --rules -f csv"
        rule_err_msg = "Rule violation"

        # create a basic parameter without a value, so the rule cannot be violated
        param1 = "param1"
        self.set_param(cmd_env, proj_name, param1, "some-value")
        self.unset_param(cmd_env, proj_name, param1)

        # see no rules
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn("No parameter rules found in project", result.out())

        set_cmd = param_cmd + f"set {param1} "
        min_len = 10
        max_len = 15
        regex = "abc.*"

        result = self.run_cli(cmd_env, set_cmd + f"--min-len {min_len} --max-len {max_len} --regex '{regex}'")
        self.assertEqual(result.return_value, 0)

        # see the 3 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},-,,string,3,static,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{param1},string,max-len,{max_len}", result.out())
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertIn(f"{param1},string,regex,{regex}", result.out())

        result = self.run_cli(cmd_env, param_cmd + "list --rules")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
+--------+------------+-----------+------------+
| Name   | Param Type | Rule Type | Constraint |
+--------+------------+-----------+------------+
| param1 | string     | max-len   | 15         |
| param1 | string     | min-len   | 10         |
| param1 | string     | regex     | abc.*      |
+--------+------------+-----------+------------+
""")

        # test min-len
        value = "a" * (min_len - 1)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(rule_err_msg, result.err())
        self.assertIn(f"Value must be at least {min_len} characters", result.err())

        # test max-len
        value = "a" * (max_len + 1)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(rule_err_msg, result.err())
        self.assertIn(f"Value must be at most {max_len} characters", result.err())

        # test regex
        value = "a" * int((max_len + max_len) / 2)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(rule_err_msg, result.err())
        self.assertIn("Value does not match regular expression", result.err())

        # something in the middle, so it is successful
        value = "abc" * int((max_len + min_len) / 6)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},string,3,static,false", result.out())

        #################
        # update the rules
        min_len = int(min_len / 2)
        result = self.run_cli(cmd_env, set_cmd + f"--min-len {min_len}")
        self.assertEqual(result.return_value, 0)

        max_len = max_len * 2
        result = self.run_cli(cmd_env, set_cmd + f"--max-len {max_len}")
        self.assertEqual(result.return_value, 0)

        regex = "a.*"
        result = self.run_cli(cmd_env, set_cmd + f"--regex '{regex}'")
        self.assertEqual(result.return_value, 0)

        # see the 3 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},string,3,static,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{param1},string,max-len,{max_len}", result.out())
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertIn(f"{param1},string,regex,{regex}", result.out())

        ################
        # remove the rules, one by one

        # regex
        result = self.run_cli(cmd_env, set_cmd + "--no-regex")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},string,2,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn(f"{param1},string,max-len,{max_len}", result.out())
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertNotIn("regex", result.out())

        result = self.run_cli(cmd_env, set_cmd + "--no-regex")
        self.assertEqual(result.return_value, 0)

        # max-len
        result = self.run_cli(cmd_env, set_cmd + "--no-max-len")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},string,1,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertNotIn("max-len", result.out())
        self.assertNotIn("regex", result.out())

        result = self.run_cli(cmd_env, set_cmd + "--no-max-len")
        self.assertEqual(result.return_value, 0)

        # min-len
        result = self.run_cli(cmd_env, set_cmd + "--no-min-len")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},string,0,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn("No parameter rules found in project", result.out())

        result = self.run_cli(cmd_env, set_cmd + "--no-min-len")
        self.assertEqual(result.return_value, 0)

        # TODO: failed create/update with values in place

        #################
        # negative tests for bad rule types: --max, and --min

        # TODO: this should not be necessary
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        result = self.run_cli(cmd_env, set_cmd + "--max -10 --min -1")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max rules not valid for string parameters", result.err())
        self.assertIn("min rules not valid for string parameters", result.err())

        result = self.run_cli(cmd_env, set_cmd + "--max -10")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max rules not valid for string parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},-,,string,0,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # see we don't leave any parameter behind when creating a parameter with an invalid rule
        self.delete_param(cmd_env, proj_name, param1)

        result = self.run_cli(cmd_env, set_cmd + "--type string --value 9 --max 10")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max rules not valid for string parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(self._empty_message(proj_name), result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

    def test_parameter_rules_integer(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-integer-rules")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("int-env")
        self.create_environment(cmd_env, env_name)
        param_cmd = base_cmd + f"--project {proj_name} --env {env_name} param "
        list_cmd = param_cmd + "ls -v -f csv"
        rules_cmd = param_cmd + "ls --rules -f csv"
        rule_err_msg = "Rule violation"

        # create a basic parameter without a value, so the rule cannot be violated
        param1 = "param1"
        self.set_param(cmd_env, proj_name, param1, "2154", param_type="integer", env=env_name)
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        # see no rules
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn("No parameter rules found in project", result.out())

        #######################
        # string stuff
        set_cmd = param_cmd + f"set {param1} "
        min_value = 1000
        max_value = 3000

        result = self.run_cli(cmd_env, set_cmd + f"--min {min_value} --max {max_value}")
        self.assertEqual(result.return_value, 0)

        # see the 2 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},-,,integer,2,static,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{param1},integer,max,{max_value}", result.out())
        self.assertIn(f"{param1},integer,min,{min_value}", result.out())

        result = self.run_cli(cmd_env, param_cmd + "list --rules")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
+--------+------------+-----------+------------+
| Name   | Param Type | Rule Type | Constraint |
+--------+------------+-----------+------------+
| param1 | integer    | max       | 3000       |
| param1 | integer    | min       | 1000       |
+--------+------------+-----------+------------+
""")

        # test min
        value = min_value - 1
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(rule_err_msg, result.err())
        self.assertIn(f"Value is less than the minimum value of {min_value}", result.err())

        # test max
        value = max_value + 1
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(rule_err_msg, result.err())
        self.assertIn(f"Value is greater than the maximum value of {max_value}", result.err())

        # something in the middle, so it is successful
        value = int((max_value + min_value) / 2)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},integer,2,static,false", result.out())

        #################
        # update the rules
        min_value = int(min_value / 2)
        result = self.run_cli(cmd_env, set_cmd + f"--min {min_value}")
        self.assertEqual(result.return_value, 0)

        max_value = max_value * 2
        result = self.run_cli(cmd_env, set_cmd + f"--max {max_value}")
        self.assertEqual(result.return_value, 0)

        # see the 2 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},integer,2,static,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{param1},integer,max,{max_value}", result.out())
        self.assertIn(f"{param1},integer,min,{min_value}", result.out())

        ################
        # remove the rules, one by one

        # max
        result = self.run_cli(cmd_env, set_cmd + "--no-max")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},integer,1,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn(f"{param1},integer,min,{min_value}", result.out())
        self.assertNotIn("max", result.out())
        self.assertNotIn("regex", result.out())

        # min
        result = self.run_cli(cmd_env, set_cmd + "--no-min")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},{value},{env_name},integer,0,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn("No parameter rules found in project", result.out())

        # TODO: failed create/update with values in place

        ################
        # negative tests for bad rule types: --max-len, --min-len, --regex

        # TODO: this should not be necessary
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        result = self.run_cli(cmd_env, set_cmd + "--max-len -10 --min-len -1 --regex 'abc.*'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max-len rules not valid for integer parameters", result.err())
        self.assertIn("min-len rules not valid for integer parameters", result.err())
        self.assertIn("regex rules not valid for integer parameters", result.err())

        result = self.run_cli(cmd_env, set_cmd + "--min-len 10")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("min-len rules not valid for integer parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},-,,integer,0,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # see we don't leave any parameter behind when creating a parameter with an invalid rule
        self.delete_param(cmd_env, proj_name, param1)

        result = self.run_cli(cmd_env, set_cmd + "--type integer --value 9 --max-len 100")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max-len rules not valid for integer parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(self._empty_message(proj_name), result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

    def test_parameter_rules_bool(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-boolean-rules")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("bool-env")
        self.create_environment(cmd_env, env_name)
        param_cmd = base_cmd + f"--project {proj_name} --env {env_name} param "
        list_cmd = param_cmd + "ls -v -f csv"
        rules_cmd = param_cmd + "ls --rules -f csv"

        # create a basic parameter without a value, so the rule cannot be violated
        param1 = "param1"
        self.set_param(cmd_env, proj_name, param1, "true", param_type="bool", env=env_name)
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        # see no rules
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertIn("No parameter rules found in project", result.out())

        set_cmd = param_cmd + f"set {param1} "

        ################
        # negative tests for bad rule types: --max, --min, --max-len, --min-len, --regex

        # TODO: this should not be necessary
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        result = self.run_cli(cmd_env, set_cmd + "--max 100 --min 10 --max-len -10 --min-len -1 --regex 'abc.*'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max rules not valid for bool parameters", result.err())
        self.assertIn("min rules not valid for bool parameters", result.err())
        self.assertIn("max-len rules not valid for bool parameters", result.err())
        self.assertIn("min-len rules not valid for bool parameters", result.err())
        self.assertIn("regex rules not valid for bool parameters", result.err())

        result = self.run_cli(cmd_env, set_cmd + "--min-len 10")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("min-len rules not valid for bool parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(f"{param1},-,,bool,0,static,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # see we don't leave any parameter behind when creating a parameter with an invalid rule
        self.delete_param(cmd_env, proj_name, param1)

        result = self.run_cli(cmd_env, set_cmd + "--type bool --value true --max 10")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("max rules not valid for bool parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertIn(self._empty_message(proj_name), result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)

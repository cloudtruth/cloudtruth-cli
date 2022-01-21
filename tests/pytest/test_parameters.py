import datetime

from typing import Dict
from typing import Tuple
from testcase import find_by_prop
from testcase import TestCase
from testcase import DEFAULT_ENV_NAME
from testcase import DEFAULT_PARAM_VALUE
from testcase import PROP_CREATED
from testcase import PROP_MODIFIED
from testcase import PROP_NAME
from testcase import PROP_VALUE
from testcase import REDACTED
from testcase import TEST_PAGE_SIZE


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
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # same result with the --values flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # same result with the --values and --secrets flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        ########
        # add first, non-secret parameter
        key1 = "my_param"
        value1 = "cRaZy value"
        desc1 = "this is just a test description"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --value '{value1}' --desc '{desc1}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | false  | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
my_param,cRaZy value,default,string,0,internal,false,this is just a test description
""")
        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(f"{value1}", result.out())

        # get the parameter details
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1} --details")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {key1}", result.out())
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn("Source: default", result.out())
        self.assertIn("Secret: false", result.out())
        self.assertIn(f"Description: {desc1}", result.out())

        # idempotent -- same value
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value1}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertIn(value1, result.out())
        self.assertIn(desc1, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(value1, result.out())

        ########
        # update the just the value
        value2 = "new_value"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value2}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertIn(value2, result.out())
        self.assertIn(desc1, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(value2, result.out())

        ########
        # rename the parameter
        orig_name = key1
        key1 = "renamed_param"
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} -r {key1}")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated parameter '{key1}'", result.out())

        ########
        # update the just the description
        desc2 = "alt description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} -d '{desc2}'")
        self.assertResultSuccess(result)

        ########
        # no updates provided
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1}")
        self.assertResultSuccess(result)
        self.assertIn("Updated parameter", result.out())

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertIn(value2, result.out())
        self.assertIn(desc2, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(value2, result.out())

        ########
        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete --yes '{key1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Removed parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # idempotent
        result = self.run_cli(cmd_env, sub_cmd + f"delete -y '{key1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Did not find parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        ###########
        # create a parameter with no value
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}'")
        self.assertResultSuccess(result)
        self.assertIn("Created parameter", result.out())
        self.assertNotIn("for environment", result.out())

        result = self.run_cli(cmd_env, sub_cmd + "list --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{key1},{DEFAULT_PARAM_VALUE},,string,0,internal,false", result.out())

        # make sure we error out on conflicting args
        mutually_exclusive = "are mutually exclusive"
        result = self.run_cli(cmd_env, sub_cmd + "list --rules --external")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --rules --evaluated")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --external --evaluated")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --parents --external")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --parents --evaluated")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --parents --rules")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --children --parents")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --children --rules")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --children --external")
        self.assertResultWarning(result, mutually_exclusive)
        result = self.run_cli(cmd_env, sub_cmd + "list --children --evaluated")
        self.assertResultWarning(result, mutually_exclusive)

        # delete the project
        self.delete_project(cmd_env, proj_name)

    def test_parameter_secret_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-secret")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # same result with the --values flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # same result with the --values and --secrets flag
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result), "Initial empty parameters"
        self.assertIn(empty_msg, result.out())

        ########
        # add first, secret parameter
        key1 = "my_param"
        value1 = "super-SENSITIVE-vAluE"
        desc1 = "my secret value"
        create_cmd = sub_cmd + f"set {key1} --secret true --value '{value1}' --desc '{desc1}'"
        result = self.run_cli(cmd_env, create_cmd)
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------+---------+------------+-------+----------+--------+-----------------+
| Name     | Value | Source  | Param Type | Rules | Type     | Secret | Description     |
+----------+-------+---------+------------+-------+----------+--------+-----------------+
| my_param | ***** | default | string     | 0     | internal | true   | my secret value |
+----------+-------+---------+------------+-------+----------+--------+-----------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
my_param,{REDACTED},default,string,0,internal,true,my secret value
""")

        # now, display with the secrets value
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-----------------------+---------+------------+-------+----------+--------+-----------------+
| Name     | Value                 | Source  | Param Type | Rules | Type     | Secret | Description     |
+----------+-----------------------+---------+------------+-------+----------+--------+-----------------+
| my_param | super-SENSITIVE-vAluE | default | string     | 0     | internal | true   | my secret value |
+----------+-----------------------+---------+------------+-------+----------+--------+-----------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets --format csv")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
my_param,super-SENSITIVE-vAluE,default,string,0,internal,true,my secret value
""")

        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(f"{value1}", result.out())

        # get the parameter details
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1} --details")
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {key1}", result.out())
        self.assertIn(f"Value: {value1}", result.out())
        self.assertIn("Source: default", result.out())
        self.assertIn("Secret: true", result.out())
        self.assertIn(f"Description: {desc1}", result.out())

        # idempotent -- same value
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value1}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertIn(value1, result.out())
        self.assertIn(desc1, result.out())

        # make sure it is still a secret
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertNotIn(value1, result.out())
        self.assertIn(REDACTED, result.out())
        self.assertIn(desc1, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(value1, result.out())

        ########
        # update the just the value
        value2 = "new_value"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value2}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertIn(value2, result.out())
        self.assertIn(desc1, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(value2, result.out())

        ########
        # update the just the description
        desc2 = "alt description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} -d '{desc2}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertResultSuccess(result)
        self.assertIn(key1, result.out())
        self.assertIn(value2, result.out())
        self.assertIn(desc2, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertResultSuccess(result)
        self.assertIn(value2, result.out())

        ########
        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete -y '{key1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Removed parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # idempotent
        result = self.run_cli(cmd_env, sub_cmd + f"delete -y '{key1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Did not find parameter '{key1}'", result.out())

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        ###########
        # create a secret with no value
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --secret true")
        self.assertResultSuccess(result)
        self.assertIn("Created parameter", result.out())
        self.assertNotIn("for environment", result.out())

        result = self.run_cli(cmd_env, sub_cmd + "list --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{key1},{DEFAULT_PARAM_VALUE},,string,0,internal,true", result.out())

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
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+-----------+------------+---------+------------+-------+----------+--------+-------------+
| Name      | Value      | Source  | Param Type | Rules | Type     | Secret | Description |
+-----------+------------+---------+------------+-------+----------+--------+-------------+
| sensitive | classified | default | string     | 0     | internal | true   |             |
| sna       | foo        | default | string     | 0     | internal | false  |             |
+-----------+------------+---------+------------+-------+----------+--------+-------------+
""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param ls -v -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+-----------+------------+---------+------------+-------+----------+--------+-------------+
| Name      | Value      | Source  | Param Type | Rules | Type     | Secret | Description |
+-----------+------------+---------+------------+-------+----------+--------+-------------+
| sensitive | top-secret | default | string     | 0     | internal | true   |             |
| sna       | fu         | default | string     | 0     | internal | false  |             |
+-----------+------------+---------+------------+-------+----------+--------+-------------+
""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name1} param export docker -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
SENSITIVE=classified
SNA=foo

""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param export docker -s")
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, proj_cmd + "param env 'no-such-parameter' -f csv")
        self.assertResultError(result, "Parameter 'no-such-parameter' was not found")

        result = self.run_cli(cmd_env, proj_cmd + f"param env {var1_name} -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name1},{var1_value1},,", result.out())
        self.assertNotIn(env_name2, result.out())

        result = self.run_cli(cmd_env, proj_cmd + f"param env {var1_name} -f csv --all")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name1},{var1_value1},,", result.out())
        self.assertIn(f"{env_name2},-,,", result.out())

        # add the parameters for the second environment
        self.set_param(cmd_env, proj_name, var1_name, var1_value2, env=env_name2)
        self.set_param(cmd_env, proj_name, var2_name, var2_value2, True, env=env_name2)

        # see that values are inherited in the different environments
        result = self.run_cli(cmd_env, env1_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value2},{env_name2}", result.out())
        self.assertIn(f"{var2_name},{var2_value2},{env_name2}", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value2},{env_name2}", result.out())
        self.assertIn(f"{var2_name},{var2_value2},{env_name2}", result.out())

        # add the parameters for the third environment
        self.set_param(cmd_env, proj_name, var1_name, var1_value3, env=env_name3)
        self.set_param(cmd_env, proj_name, var2_name, var2_value3, True, env=env_name3)

        # see that secrets do not show up without -s
        result = self.run_cli(cmd_env, proj_cmd + f"param environment '{var2_name}' -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name1},{REDACTED},,", result.out())
        self.assertIn(f"{env_name2},{REDACTED},,", result.out())
        self.assertIn(f"{env_name3},{REDACTED},,", result.out())

        # see that secrets do not show up without -s
        result = self.run_cli(cmd_env, proj_cmd + f"param environment '{var2_name}' -f csv -s")
        self.assertResultSuccess(result)
        self.assertIn(f"{env_name1},{var2_value1},,", result.out())
        self.assertIn(f"{env_name2},{var2_value2},,", result.out())
        self.assertIn(f"{env_name3},{var2_value3},,", result.out())

        # see that values are inherited in the different environments
        result = self.run_cli(cmd_env, env1_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())
        self.assertIn(f"{var2_name},{var2_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value2},{env_name2}", result.out())
        self.assertIn(f"{var2_name},{var2_value2},{env_name2}", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value3},{env_name3}", result.out())
        self.assertIn(f"{var2_name},{var2_value3},{env_name3}", result.out())

        docker_cmd = " param export docker -s"
        result = self.run_cli(cmd_env, proj_cmd + docker_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
{var1_name.upper()}={var1_value1}
{var2_name.upper()}={var2_value1}

""")

        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name2}" + docker_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
{var1_name.upper()}={var1_value2}
{var2_name.upper()}={var2_value2}

""")

        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name3}" + docker_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
{var1_name.upper()}={var1_value3}
{var2_name.upper()}={var2_value3}

""")

        # remove env2 override
        unset_cmd = f"param unset '{var1_name}'"
        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name2} " + unset_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Removed parameter value '{var1_name}'", result.out())
        self.assertIn(f"for environment '{env_name2}'", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value3},{env_name3}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env1_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        # remove env3 override
        result = self.run_cli(cmd_env, proj_cmd + f"--env {env_name3} " + unset_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Removed parameter value '{var1_name}'", result.out())
        self.assertIn(f"for environment '{env_name3}'", result.out())

        result = self.run_cli(cmd_env, env3_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env2_list)
        self.assertResultSuccess(result)
        self.assertIn(f"{var1_name},{var1_value1},{env_name1}", result.out())

        result = self.run_cli(cmd_env, env1_list)
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

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
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET={REDACTED}
SECOND_PARAM=a value with spaces
SECOND_SECRET={REDACTED}

""")

        result = self.run_cli(cmd_env, docker_cmd + "--secrets")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces
SECOND_SECRET=sensitive value with spaces

""")

        result = self.run_cli(cmd_env, docker_cmd + "--secrets --starts-with SECOND")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
SECOND_PARAM=a value with spaces
SECOND_SECRET=sensitive value with spaces

""")

        # use uppercase key without secrets
        result = self.run_cli(cmd_env, docker_cmd + "--starts-with FIRST")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET={REDACTED}

""")

        # use uppercase key with secrets
        result = self.run_cli(cmd_env, docker_cmd + "--starts-with FIRST -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci

""")

        # use lowercase key with secrets
        result = self.run_cli(cmd_env, docker_cmd + "--contains param -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces

""")

        # see if filter picks up non-posix
        result = self.run_cli(cmd_env, docker_cmd + "--contains posix -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\

""")

        #####################
        # Dotenv
        dotenv_cmd = base_cmd + f"--project {proj_name} param export dotenv "
        result = self.run_cli(cmd_env, dotenv_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM="posix_compliant_value"
FIRST_PARAM_SECRET="{REDACTED}"
SECOND_PARAM="a value with spaces"
SECOND_SECRET="{REDACTED}"

""")

        dotenv_cmd = base_cmd + f"--project {proj_name} param export dotenv -s"
        result = self.run_cli(cmd_env, dotenv_cmd)
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
FIRST_PARAM=posix_compliant_value
FIRST_PARAM_SECRET={REDACTED}
SECOND_PARAM='a value with spaces'
SECOND_SECRET={REDACTED}

""")

        shell_cmd = base_cmd + f"--project {proj_name} param export shell -s"
        result = self.run_cli(cmd_env, shell_cmd)
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result), "Initial empty parameters"
        self.assertIn(empty_msg, result.out())

        ########
        # add first, non-secret parameter
        key1 = "my_param"
        value1 = "cRaZy value"
        desc1 = "this is just a test description"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --value '{value1}' --desc '{desc1}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | false  | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
""")

        # switch it to a secret
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret true")
        self.assertResultSuccess(result)

        # see that it has been changed to a secret (redacted in cli)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | ***** | default | string     | 0     | internal | true   | this is just a test description |
+----------+-------+---------+------------+-------+----------+--------+---------------------------------+
""")

        # verify value has not changed
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | true   | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
""")

        # switch back to a regular parameter
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret false")
        self.assertResultSuccess(result)

        # see that it is no longer redacted
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| Name     | Value       | Source  | Param Type | Rules | Type     | Secret | Description                     |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
| my_param | cRaZy value | default | string     | 0     | internal | false  | this is just a test description |
+----------+-------------+---------+------------+-------+----------+--------+---------------------------------+
""")

        self.delete_project(cmd_env, proj_name)

    def test_parameter_local_file(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # create the file with the value
        filename = self.make_name("value")
        value1 = "static val from file"
        self.write_file(filename, value1)

        # add a new project
        proj_name = self.make_name("test-local-file")
        empty_msg = self._empty_message(proj_name)
        self.create_project(cmd_env, proj_name)

        # check that there are no parameters
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result), "Initial empty parameters"
        self.assertIn(empty_msg, result.out())

        ########
        # add first, non-secret parameter from file
        key1 = "my_param"
        desc1 = "param set from file input"
        result = self.run_cli(cmd_env,
                              sub_cmd + f"set {key1} --input '{filename}' --desc '{desc1}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+----------------------+---------+------------+-------+----------+--------+---------------------------+
| Name     | Value                | Source  | Param Type | Rules | Type     | Secret | Description               |
+----------+----------------------+---------+------------+-------+----------+--------+---------------------------+
| my_param | static val from file | default | string     | 0     | internal | false  | param set from file input |
+----------+----------------------+---------+------------+-------+----------+--------+---------------------------+
""")

        # change value from `--value` flag from CLI
        value2 = "update-from-value"
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --value '{value2}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+-------------------+---------+------------+-------+----------+--------+---------------------------+
| Name     | Value             | Source  | Param Type | Rules | Type     | Secret | Description               |
+----------+-------------------+---------+------------+-------+----------+--------+---------------------------+
| my_param | update-from-value | default | string     | 0     | internal | false  | param set from file input |
+----------+-------------------+---------+------------+-------+----------+--------+---------------------------+
""")

        # update with a different value from file
        value3 = "another-static-file"
        self.write_file(filename, value3)

        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --input '{filename}'")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+----------+---------------------+---------+------------+-------+----------+--------+---------------------------+
| Name     | Value               | Source  | Param Type | Rules | Type     | Secret | Description               |
+----------+---------------------+---------+------------+-------+----------+--------+---------------------------+
| my_param | another-static-file | default | string     | 0     | internal | false  | param set from file input |
+----------+---------------------+---------+------------+-------+----------+--------+---------------------------+
""")

        # cleanup
        self.delete_file(filename)
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
        self.assertResultSuccess(result), "Initial empty parameters"
        self.assertIn(empty_msg, result.out())

        key1 = "param1"
        value1 = "value"
        fqn = "GitHub::bogus::repo::directory::file"
        jmes = "foo.bar"
        conflict_msg = "Conflicting arguments: cannot specify"
        invalid_fqn_msg = "Invalid FQN or JMES path expression"
        invalid_fqn_msg = "No integration provider available for"

        #####################
        # verify over specifying
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' -v '{value1}' --fqn '{fqn}'")
        self.assertResultError(result, conflict_msg)

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --prompt --fqn '{fqn}'")
        self.assertResultError(result, conflict_msg)

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --input 'missing.txt' --fqn '{fqn}'")
        self.assertResultError(result, conflict_msg)

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --prompt --jmes '{jmes}'")
        self.assertResultError(result, conflict_msg)

        # check that nothing was added
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        #####################
        # poorly structured FQN
        completely_bogus_msg = "missing the network location"
        completely_bogus_msg = "No integration provider available for"

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        self.assertResultError(result, invalid_fqn_msg)
        self.assertResultError(result, completely_bogus_msg)

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        self.assertResultError(result, invalid_fqn_msg)
        self.assertResultError(result, completely_bogus_msg)

        #####################
        # no such FQN provider
        fqn = "foobar://bogus::repo::directory::file"
        no_provider_msg = "No integration provider available for"

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        self.assertResultError(result, invalid_fqn_msg)
        self.assertResultError(result, no_provider_msg)

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        self.assertResultError(result, invalid_fqn_msg)
        self.assertResultError(result, no_provider_msg)

        #####################
        # no such FQN, but a legit provider
        fqn = "github://this-is-a-crazy/repo-path/that/does/not/exist"
        no_integration_msg = "No integration available for"

        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        # self.assertResultError(result, invalid_fqn_msg)
        self.assertResultError(result, no_integration_msg)

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        # self.assertResultError(result, invalid_fqn_msg)
        self.assertResultError(result, no_integration_msg)

        # check that nothing was added
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets -f csv")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # verify `--external` flag causes specialized warning
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        empty_msg = f"No external parameters found in project {proj_name}"
        result = self.run_cli(cmd_env, sub_cmd + "list --external")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "list --external -v")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "list --external -v -s")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "list --external -v -s --show-times")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # test backward compatibility (--dynamic flag still works)
        result = self.run_cli(cmd_env, sub_cmd + "list --dynamic -v -s --show-times")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

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
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

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
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| Name      | Value                          | Source  | Param Type | Rules | Type     | Secret | Description |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| speicla14 | *****                          | default | string     | 0     | internal | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | string     | 0     | internal | false  | Jade lunch  |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| Name      | Value                          | Source  | Param Type | Rules | Type     | Secret | Description |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
| speicla14 | cueey-chicken                  | default | string     | 0     | internal | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | string     | 0     | internal | false  | Jade lunch  |
+-----------+--------------------------------+---------+------------+-------+----------+--------+-------------+
""")

        #################
        # CSV format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
speicla14,{REDACTED},default,string,0,internal,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,string,0,internal,false,Jade lunch
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
Name,Value,Source,Param Type,Rules,Type,Secret,Description
speicla14,cueey-chicken,default,string,0,internal,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,string,0,internal,false,Jade lunch
""")

        #################
        # JSON format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f json")
        self.assertResultSuccess(result)
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
      "Type": "internal",
      "Value": "*****"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "internal",
      "Value": "beef brocolli, pork fried rice"
    }
  ]
}
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f json -s")
        self.assertResultSuccess(result)
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
      "Type": "internal",
      "Value": "cueey-chicken"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
      "Param Type": "string",
      "Rules": "0",
      "Secret": "false",
      "Source": "default",
      "Type": "internal",
      "Value": "beef brocolli, pork fried rice"
    }
  ]
}
""")

        #################
        # YAML format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f yaml")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
---
parameter:
  - Description: Jade secret
    Name: speicla14
    Param Type: string
    Rules: "0"
    Secret: "true"
    Source: default
    Type: internal
    Value: "{REDACTED}"
  - Description: Jade lunch
    Name: speicla3
    Param Type: string
    Rules: "0"
    Secret: "false"
    Source: default
    Type: internal
    Value: "beef brocolli, pork fried rice"
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f yaml -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), """\
---
parameter:
  - Description: Jade secret
    Name: speicla14
    Param Type: string
    Rules: "0"
    Secret: "true"
    Source: default
    Type: internal
    Value: cueey-chicken
  - Description: Jade lunch
    Name: speicla3
    Param Type: string
    Rules: "0"
    Secret: "false"
    Source: default
    Type: internal
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
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

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
            result = self.run_cli(cmd_env, sub_cmd + f"set -r '{temp_name}' '{param_name}'")
            self.assertResultSuccess(result)
            self.verify_param(cmd_env, proj_name, temp_name, param_value)

            # back to the original name
            result = self.run_cli(cmd_env, sub_cmd + f"set -r '{param_name}' '{temp_name}'")
            self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param1},{value1a},{DEFAULT_PARAM_VALUE}
{param2},{REDACTED},{DEFAULT_PARAM_VALUE}
""")

        result = self.run_cli(cmd_env, diff_cmd + "-s")
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Parameter,{env_a},{env_b}
{param1},{value1a},{value1d}
{param2},{value2a},{value2d}
""")

        # now, let's see the properties
        result = self.run_cli(cmd_env, diff_cmd + "-s -p value -p environment ")
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertIn("No parameters or differences in compared properties found", result.out())

        #####################
        # Time diff
        diff_csv = sub_cmd + "diff -f csv "

        # test single time
        result = self.run_cli(cmd_env, diff_csv + f"--as-of '{modified_a}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Parameter,Current,{modified_a}
{param1},{value1d},-
{param2},{REDACTED},-
""")

        # compare 2 points in time (with secrets)
        result = self.run_cli(cmd_env, diff_csv + f"-s --as-of '{modified_a}' --as-of  '{modified_b}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Parameter,{modified_a},{modified_b}
{param1},-,{value1d}
{param2},-,{value2d}
""")

        # compare 2 points in time where there are no differences
        result = self.run_cli(cmd_env, diff_csv + f"--as-of '{created_a}' --as-of '{modified_a}'")
        self.assertResultSuccess(result)
        self.assertIn("No parameters or differences in compared properties found", result.out())

        #####################
        # Combination environment/time diff

        # if just one env/time, it applies to the right hand side
        result = self.run_cli(cmd_env, diff_csv + f"-e '{env_a}' --as-of '{modified_a}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Parameter,default,{env_a} ({modified_a})
{param1},{value1d},{value1a}
""")

        # the full set of environments/times (with secrets)
        cmd = diff_csv + f"-e '{env_a}' --as-of '{modified_a}' -e '{env_b}' --as-of '{modified_b}' -s"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
Parameter,{env_a} ({modified_a}),{env_b} ({modified_b})
{param1},{value1a},{same}
{param2},{value2a},{value2b}
""")

        #####################
        # Error cases

        # no comparing to yourself
        result = self.run_cli(cmd_env, sub_cmd + "difference")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        matched_envs = f"-e '{env_a}' " * 2
        result = self.run_cli(cmd_env, sub_cmd + f"difference {matched_envs}")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        matched_times = "--as-of 2021-08-27 " * 2
        result = self.run_cli(cmd_env, sub_cmd + f"difference {matched_times}")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        result = self.run_cli(cmd_env, sub_cmd + f"difference {matched_times} {matched_envs}")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        # first environment DNE
        result = self.run_cli(cmd_env, sub_cmd + "differ -e 'charlie-foxtrot' -e '{env_b}'")
        self.assertResultError(result, "Did not find environment 'charlie-foxtrot'")

        # second environment DNE
        result = self.run_cli(cmd_env, sub_cmd + f"differences -e '{env_a}' -e 'missing'")
        self.assertResultError(result, "Did not find environment 'missing'")

        # too many specified
        result = self.run_cli(cmd_env, sub_cmd + "diff -e env1 --env env2 -e env3")
        self.assertResultWarning(result, "Can specify a maximum of 2 environment values")

        result = self.run_cli(cmd_env, sub_cmd + "diff --as-of 2021-08-01 --as-of 2021-08-02 --as-of 2021-08-03")
        self.assertResultWarning(result, "Can specify a maximum of 2 as-of values")

        # cleanup
        self.delete_environment(cmd_env, env_a)
        self.delete_environment(cmd_env, env_b)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_as_of_time(self):
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
        self.verify_param(cmd_env, proj_name, param1, value_a1, env=env_a, as_of=details_a1.get(PROP_MODIFIED))
        self.verify_param(cmd_env, proj_name, param1, value_b1, env=env_b, as_of=details_b1.get(PROP_MODIFIED))

        self.verify_param(cmd_env, proj_name, param1, value_a2, env=env_a, as_of=details_a2.get(PROP_MODIFIED))
        self.verify_param(cmd_env, proj_name, param1, value_b2, env=env_b, as_of=details_b2.get(PROP_MODIFIED))

        # check with details turned on
        cmd = base_cmd + f"--project '{proj_name}' --env '{env_a}' param get '{param1}' -d"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{PROP_CREATED}: {details_a2.get(PROP_CREATED)}", result.out())
        self.assertIn(f"{PROP_MODIFIED}: {details_a2.get(PROP_MODIFIED)}", result.out())
        self.assertIn(f"{PROP_VALUE}: {details_a2.get(PROP_VALUE)}", result.out())

        # time filtered with details with other environment
        cmd = base_cmd + f"--project '{proj_name}' --env '{env_b}' param get '{param1}' -d --as-of {modified_at}"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{PROP_CREATED}: {details_b1.get(PROP_CREATED)}", result.out())
        self.assertIn(f"{PROP_MODIFIED}: {details_b1.get(PROP_MODIFIED)}", result.out())
        self.assertIn(f"{PROP_VALUE}: {details_b1.get(PROP_VALUE)}", result.out())

        ####################
        # verify the 'list' command returns the correct values
        # NOTE: this leverages the get_param(), since it uses the 'param list' command
        self.assertEqual(details_a2, self.get_param(cmd_env, proj_name, param1, env=env_a))
        self.assertEqual(details_b2, self.get_param(cmd_env, proj_name, param1, env=env_b))

        self.assertEqual(details_a1, self.get_param(cmd_env, proj_name, param1, env=env_a, as_of=modified_at))
        self.assertEqual(details_b1, self.get_param(cmd_env, proj_name, param1, env=env_b, as_of=modified_at))

        ####################
        # verify the 'environments' command returns the correct values
        def equal_properties(entry: Dict, details: Dict) -> bool:
            return (entry.get(PROP_VALUE) == details.get(PROP_VALUE)
                    and entry.get(PROP_CREATED) == details.get(PROP_CREATED)
                    and entry.get(PROP_MODIFIED) == details.get(PROP_MODIFIED))

        param_cmd = base_cmd + f"--project '{proj_name}' param "
        env_cmd = param_cmd + f"env '{param1}' --show-times --format json "
        result = self.run_cli(cmd_env, env_cmd)
        self.assertResultSuccess(result)
        data = eval(result.out())
        for item in data["parameter"]:
            if item.get("Environment") == env_a:
                self.assertTrue(equal_properties(item, details_a2))
            if item.get("Environment") == env_b:
                self.assertTrue(equal_properties(item, details_b2))

        env_cmd += f"--as-of {modified_at}"
        result = self.run_cli(cmd_env, env_cmd)
        self.assertResultSuccess(result)
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
        self.assertResultSuccess(result)
        self.assertIn(f"{param1.upper()}={value_a2}", result.out())

        result = self.run_cli(cmd_env, export_cmd + f" --as-of {modified_at}")
        self.assertResultSuccess(result)
        self.assertIn(f"{param1.upper()}={value_a1}", result.out())

        ####################
        # negative cases with times before when the project was created
        timestamp = "3/24/2021"
        no_project = "No HistoricalProject matches the given query"
        param_cmd = base_cmd + f"--project '{proj_name}' param "
        result = self.run_cli(cmd_env, param_cmd + f"ls -v --as-of '{timestamp}'")
        self.assertResultError(result, no_project)
        result = self.run_cli(cmd_env, param_cmd + f"diff --as-of '{timestamp}'")
        self.assertResultError(result, no_project)
        result = self.run_cli(cmd_env, param_cmd + f"env '{param1}' --as-of '{timestamp}'")
        self.assertResultError(result, no_project)

        no_environment = "No HistoricalEnvironment matches the given query"
        result = self.run_cli(cmd_env, param_cmd + f"export shell --as-of '{timestamp}'")
        self.assertResultError(result, no_environment)

        result = self.run_cli(cmd_env, param_cmd + f"get '{param1}' --as-of '{timestamp}'")
        self.assertResultError(result, no_project)

        # cleanup
        self.delete_environment(cmd_env, env_a)
        self.delete_environment(cmd_env, env_b)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_evaluated(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project, and a couple environments
        proj_name = self.make_name("test-evaluated")
        self.create_project(cmd_env, proj_name)
        env_name_a = self.make_name("test-eval-a")
        self.create_environment(cmd_env, env_name_a)
        env_name_b = self.make_name("test-eval-b")
        self.create_environment(cmd_env, env_name_b)

        # create an internal parameter
        param1 = "param1"
        value1a = "first value"
        value1b = "other value"
        self.set_param(cmd_env, proj_name, param1, value1a, env=env_name_a)
        self.set_param(cmd_env, proj_name, param1, value1b, env=env_name_b)

        # verify `--evaluated` flag causes specialized warning
        sub_cmd = base_cmd + f" --project {proj_name} parameters "
        empty_msg = f"No evaluated parameters found in project {proj_name}"
        result = self.run_cli(cmd_env, sub_cmd + "list --evaluated")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "list --evaluated -v -s --show-times")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # create another parameter -- keep one value evaluated and the other not (even though
        # nothing to evaluate) to prove it is a value property
        param2 = "param2"
        value2a = "my-value"
        value2b = "your-value"
        self.set_param(cmd_env, proj_name, param2, value2a, env=env_name_a, evaluate=True)
        self.set_param(cmd_env, proj_name, param2, value2b, env=env_name_b, evaluate=False)

        csv_unevaluated = "string,0,internal,false,"
        csv_evaluated = "string,0,internal-evaluated,false"
        result = self.list_params(cmd_env, proj_name, env=env_name_a, fmt="csv")
        self.assertIn(f"{param1},{value1a},{env_name_a},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2a},{env_name_a},{csv_evaluated}", result.out())

        result = self.list_params(cmd_env, proj_name, env=env_name_b, fmt="csv")
        self.assertIn(f"{param1},{value1b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2b},{env_name_b},{csv_unevaluated}", result.out())

        ####################
        # add a "real" evaluated parameter
        param3 = "param3"
        value3a = f"{{{{ {param1} }}}}"
        value3b = f"{{{{ {param2} }}}}"
        self.set_param(cmd_env, proj_name, param3, value3a, env=env_name_a, evaluate=True)
        self.set_param(cmd_env, proj_name, param3, value3b, env=env_name_b, evaluate=True)

        # in env-a, the value points at param1
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj_name}' --env '{env_name_a}' param get '{param3}'")
        self.assertResultSuccess(result)
        self.assertIn(value1a, result.out())

        result = self.list_params(cmd_env, proj_name, env=env_name_a, fmt="csv")
        self.assertIn(f"{param1},{value1a},{env_name_a},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2a},{env_name_a},{csv_evaluated}", result.out())
        self.assertIn(f"{param3},{value1a},{env_name_a},{csv_evaluated}", result.out())

        result = self.list_params(cmd_env, proj_name, env=env_name_b, fmt="csv")
        self.assertIn(f"{param1},{value1b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param3},{value2b},{env_name_b},{csv_evaluated}", result.out())

        ##################
        # recursion: param4={{param3}} => value1a
        param4 = "param4"
        value4a = f"{{{{ {param3} }}}}"
        self.set_param(cmd_env, proj_name, param4, value=value4a, env=env_name_a, evaluate=True)

        result = self.list_params(cmd_env, proj_name, env=env_name_a, show_values=True, show_evaluated=True, fmt="csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{param4},{value1a},{value4a}", result.out())

        ##################
        # invalid parameter value -- see that value does not get updated
        bad_param = "cloudtruth.parameters.unknown"
        param_a_cmd = base_cmd + f"--project '{proj_name}' --env '{env_name_a}' param "
        missing_cmd = param_a_cmd + f"set '{param3}' --value '{{{{ {bad_param} }}}}'"
        result = self.run_cli(cmd_env, missing_cmd)
        self.assertResultError(result, "Evaluation error")
        self.assertIn("reference automatic parameter that does not exist", result.err())
        self.assertIn(bad_param, result.err())

        result = self.list_params(cmd_env, proj_name, env=env_name_a, fmt="csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{param3},{value1a},{env_name_a},{csv_evaluated}", result.out())

        ####################
        # set param3 to not evaluate in one environment, but verify it evaluates in the other
        self.set_param(cmd_env, proj_name, param3, env=env_name_a, evaluate=False)

        result = self.list_params(cmd_env, proj_name, env=env_name_a, fmt="csv")
        self.assertIn(f"{param1},{value1a},{env_name_a},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2a},{env_name_a},{csv_evaluated}", result.out())
        self.assertIn(f"{param3},{value3a},{env_name_a},{csv_unevaluated}", result.out())
        self.assertIn(f"{param4},{value3a},{env_name_a},{csv_evaluated}", result.out())

        result = self.list_params(cmd_env, proj_name, env=env_name_b, fmt="csv")
        self.assertIn(f"{param1},{value1b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param3},{value2b},{env_name_b},{csv_evaluated}", result.out())
        self.assertIn(f"{param4},{DEFAULT_PARAM_VALUE},,{csv_unevaluated}", result.out())

        detail_cmd = f"param get '{param3}' --details"
        get_cmd = base_cmd + f"--project '{proj_name}' --env '{env_name_a}' " + detail_cmd
        result = self.run_cli(cmd_env, get_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {param3}", result.out())
        self.assertIn(f"Value: {{{{ {param1} }}}}", result.out())
        self.assertIn(f"Source: {env_name_a}", result.out())
        self.assertIn("Evaluated: false", result.out())

        get_cmd = base_cmd + f"--project '{proj_name}' --env '{env_name_b}' " + detail_cmd
        result = self.run_cli(cmd_env, get_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Name: {param3}", result.out())
        self.assertIn(f"Value: {value2b}", result.out())
        self.assertIn(f"Source: {env_name_b}", result.out())
        self.assertIn("Evaluated: true", result.out())
        self.assertIn(f"Raw: {{{{ {param2} }}}}", result.out())

        ####################
        # set param3 no longer evaluated
        self.set_param(cmd_env, proj_name, param3, env=env_name_b, evaluate=False)

        result = self.list_params(cmd_env, proj_name, env=env_name_a, fmt="csv")
        self.assertIn(f"{param1},{value1a},{env_name_a},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2a},{env_name_a},{csv_evaluated}", result.out())
        self.assertIn(f"{param3},{value3a},{env_name_a},{csv_unevaluated}", result.out())
        self.assertIn(f"{param4},{value3a},{env_name_a},{csv_evaluated}", result.out())

        result = self.list_params(cmd_env, proj_name, env=env_name_b, fmt="csv")
        self.assertIn(f"{param1},{value1b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param2},{value2b},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param3},{{{{ {param2} }}}},{env_name_b},{csv_unevaluated}", result.out())
        self.assertIn(f"{param4},{DEFAULT_PARAM_VALUE},,{csv_unevaluated}", result.out())

        ##################
        # use one of the "pre-defined" values
        value3c = "{{ cloudtruth.environment }}"
        self.set_param(cmd_env, proj_name, param3, value=value3c, env=env_name_a, evaluate=True)
        result = self.list_params(cmd_env, proj_name, env=env_name_a, show_evaluated=True, fmt="csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{param2},{value2a},{value2a}", result.out())
        self.assertIn(f"{param3},{env_name_a},{value3c}", result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name_a)
        self.delete_environment(cmd_env, env_name_b)

    def test_parameter_types(self):
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
        bool_type = "boolean"

        result = self.run_cli(cmd_env, param_cmd + f"set {bool_param} -t {bool_type} -v {bool_value}")
        self.assertResultSuccess(result)
        self.assertIn(f"Set parameter '{bool_param}'", result.out())

        # see it in the display
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{bool_param},{bool_value},default,{bool_type},0,internal,false", result.out())

        # try to set value to non-bool value
        result = self.run_cli(cmd_env, param_cmd + f"set {bool_param} -v not-a-bool")
        self.assertResultError(result, type_err_msg)
        self.assertIn("Value is not of type bool", result.err())

        # change the type back to string
        self.set_param(cmd_env, proj_name, bool_param, bool_value, param_type="string")
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{bool_param},{bool_value},default,string,0,internal,false", result.out())

        # update back to bool
        self.set_param(cmd_env, proj_name, bool_param, bool_value, param_type=bool_type)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{bool_param},{bool_value},default,{bool_type},0,internal,false", result.out())

        # toggle to secret
        self.set_param(cmd_env, proj_name, bool_param, bool_value, secret=True)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{bool_param},{REDACTED},default,{bool_type},0,internal,true", result.out())

        # toggle back from secret
        self.set_param(cmd_env, proj_name, bool_param, bool_value, secret=False)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{bool_param},{bool_value},default,{bool_type},0,internal,false", result.out())

        #####################
        # integer tests
        int_param = "param2"
        int_value = "-1234"
        int_type = "integer"

        result = self.run_cli(cmd_env, param_cmd + f"set {int_param} -t {int_type} -v {int_value}")
        self.assertResultSuccess(result)
        self.assertIn(f"Set parameter '{int_param}'", result.out())

        # see it in the display
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{int_param},{int_value},default,{int_type},0,internal,false", result.out())

        # try to set value to non-integer value
        result = self.run_cli(cmd_env, param_cmd + f"set {int_param} -v not-an-integer")
        self.assertResultError(result, type_err_msg)
        self.assertResultError(result, "Value is not of type integer")

        # change the type back to string
        self.set_param(cmd_env, proj_name, int_param, int_value, param_type="string")
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{int_param},{int_value},default,string,0,internal,false", result.out())

        # update back to integer
        self.set_param(cmd_env, proj_name, int_param, int_value, param_type=int_type)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{int_param},{int_value},default,{int_type},0,internal,false", result.out())

        # toggle to secret
        self.set_param(cmd_env, proj_name, int_param, int_value, secret=True)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{int_param},{REDACTED},default,{int_type},0,internal,true", result.out())

        # toggle back from secret
        self.set_param(cmd_env, proj_name, int_param, int_value, secret=False)
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{int_param},{int_value},default,{int_type},0,internal,false", result.out())

        # NOTE: no real need to test 'string' types, since that is the default and no illegal values

        unknown_param = "param3"
        result = self.run_cli(cmd_env, param_cmd + f"set {unknown_param} --type foo")
        self.assertResultError(result, "Not Found (404): No ParameterType matches the given query.")

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertNotIn(unknown_param, result.out())

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
        rules_cmd = param_cmd + "ls --rules -v -f csv"
        rule_err_msg = "Rule violation"
        create_err_msg = "Rule create error"

        # create a basic parameter without a value, so the rule cannot be violated
        param1 = "param1"
        self.set_param(cmd_env, proj_name, param1, "some-value")
        self.unset_param(cmd_env, proj_name, param1)

        # see no rules
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        set_cmd = param_cmd + f"set {param1} "
        min_len = 10
        max_len = 15
        regex = "abc.*"

        result = self.run_cli(cmd_env, set_cmd + f"--min-len {min_len} --max-len {max_len} --regex '{regex}'")
        self.assertResultSuccess(result)

        # see the 3 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},-,,string,3,internal,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},string,max-len,{max_len}", result.out())
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertIn(f"{param1},string,regex,{regex}", result.out())

        result = self.run_cli(cmd_env, param_cmd + "list --rules")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"{param1}\n")

        result = self.run_cli(cmd_env, param_cmd + "list --rules -v")
        self.assertResultSuccess(result)
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
        self.assertResultError(result, rule_err_msg)
        self.assertResultError(result, f"Value must be at least {min_len} characters")

        # test max-len
        value = "a" * (max_len + 1)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertResultError(result, rule_err_msg)
        self.assertResultError(result, f"Value must be at most {max_len} characters")

        # test regex
        value = "a" * int((max_len + max_len) / 2)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertResultError(result, rule_err_msg)
        self.assertIn("Value does not match regular expression", result.err())

        # something in the middle, so it is successful
        value = "abc" * int((max_len + min_len) / 6)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},string,3,internal,false", result.out())

        #################
        # update the rules
        min_len = int(min_len / 2)
        result = self.run_cli(cmd_env, set_cmd + f"--min-len {min_len}")
        self.assertResultSuccess(result)

        max_len = max_len * 2
        result = self.run_cli(cmd_env, set_cmd + f"--max-len {max_len}")
        self.assertResultSuccess(result)

        regex = "a.*"
        result = self.run_cli(cmd_env, set_cmd + f"--regex '{regex}'")
        self.assertResultSuccess(result)

        # see the 3 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},string,3,internal,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},string,max-len,{max_len}", result.out())
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertIn(f"{param1},string,regex,{regex}", result.out())

        ################
        # remove the rules, one by one

        # regex
        result = self.run_cli(cmd_env, set_cmd + "--no-regex")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},string,2,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},string,max-len,{max_len}", result.out())
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertNotIn("regex", result.out())

        result = self.run_cli(cmd_env, set_cmd + "--no-regex")
        self.assertResultSuccess(result)

        # max-len
        result = self.run_cli(cmd_env, set_cmd + "--no-max-len")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},string,1,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},string,min-len,{min_len}", result.out())
        self.assertNotIn("max-len", result.out())
        self.assertNotIn("regex", result.out())

        result = self.run_cli(cmd_env, set_cmd + "--no-max-len")
        self.assertResultSuccess(result)

        # min-len
        result = self.run_cli(cmd_env, set_cmd + "--no-min-len")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},string,0,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        result = self.run_cli(cmd_env, set_cmd + "--no-min-len")
        self.assertResultSuccess(result)

        #################
        # failed create rules with values in place
        curr_len = len(value)
        result = self.run_cli(cmd_env, set_cmd + f"--min-len {curr_len + 3}")
        self.assertResultError(result, f"Rule create error: Rule may not be applied to {param1}")
        self.assertIn("Value must be at least", result.err())

        result = self.run_cli(cmd_env, set_cmd + f"--max-len {curr_len - 2}")
        self.assertResultError(result, f"Rule create error: Rule may not be applied to {param1}")
        self.assertIn("Value must be at most", result.err())

        #################
        result = self.run_cli(cmd_env, set_cmd + f"--min-len {curr_len - 10}")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, set_cmd + f"--min-len {curr_len + 3}")
        self.assertResultError(result, f"Rule update error: Rule may not be applied to {param1}")
        self.assertIn("Value must be at least", result.err())

        result = self.run_cli(cmd_env, set_cmd + f"--max-len {curr_len + 10}")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, set_cmd + f"--max-len {curr_len - 2}")
        self.assertResultError(result, f"Rule update error: Rule may not be applied to {param1}")
        self.assertIn("Value must be at most", result.err())

        # remove the rules
        result = self.run_cli(cmd_env, set_cmd + "--no-min-len")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, set_cmd + "--no-max-len")
        self.assertResultSuccess(result)

        #################
        # negative tests for bad rule types: --max, and --min
        result = self.run_cli(cmd_env, set_cmd + "--max -10 --min -1")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max rules not valid for string parameters", result.err())
        self.assertIn("min rules not valid for string parameters", result.err())

        result = self.run_cli(cmd_env, set_cmd + "--max -10")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max rules not valid for string parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},string,0,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # see we don't leave any parameter behind when creating a parameter with an invalid rule
        self.delete_param(cmd_env, proj_name, param1)

        result = self.run_cli(cmd_env, set_cmd + "--type string --value 9 --max 10")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max rules not valid for string parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
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
        rules_cmd = param_cmd + "ls --rules -vf csv"
        rule_err_msg = "Rule violation"
        create_err_msg = "Rule create error"

        # create a basic parameter without a value, so the rule cannot be violated
        param1 = "param1"
        self.set_param(cmd_env, proj_name, param1, "2154", param_type="integer", env=env_name)
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        # see no rules
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        #######################
        # string stuff
        set_cmd = param_cmd + f"set {param1} "
        min_value = 1000
        max_value = 3000

        result = self.run_cli(cmd_env, set_cmd + f"--min {min_value} --max {max_value}")
        self.assertResultSuccess(result)

        # see the 2 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},-,,integer,2,internal,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},integer,max,{max_value}", result.out())
        self.assertIn(f"{param1},integer,min,{min_value}", result.out())

        result = self.run_cli(cmd_env, param_cmd + "list --rules")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"{param1}\n")

        result = self.run_cli(cmd_env, param_cmd + "list --rules -v")
        self.assertResultSuccess(result)
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
        self.assertResultError(result, rule_err_msg)
        self.assertIn(f"Value is less than the minimum value of {min_value}", result.err())

        # test max
        value = max_value + 1
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertResultError(result, rule_err_msg)
        self.assertIn(f"Value is greater than the maximum value of {max_value}", result.err())

        # something in the middle, so it is successful
        value = int((max_value + min_value) / 2)
        result = self.run_cli(cmd_env, set_cmd + f"-v {value}")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},integer,2,internal,false", result.out())

        #################
        # update the rules
        min_value = int(min_value / 2)
        result = self.run_cli(cmd_env, set_cmd + f"--min {min_value}")
        self.assertResultSuccess(result)

        max_value = max_value * 2
        result = self.run_cli(cmd_env, set_cmd + f"--max {max_value}")
        self.assertResultSuccess(result)

        # see the 2 rules are registered
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},integer,2,internal,false", result.out())

        # check the --rules output (csv)
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},integer,max,{max_value}", result.out())
        self.assertIn(f"{param1},integer,min,{min_value}", result.out())

        ################
        # remove the rules, one by one

        # max
        result = self.run_cli(cmd_env, set_cmd + "--no-max")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},integer,1,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},integer,min,{min_value}", result.out())
        self.assertNotIn("max", result.out())
        self.assertNotIn("regex", result.out())

        # min
        result = self.run_cli(cmd_env, set_cmd + "--no-min")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},integer,0,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # failed create rules with values in place
        result = self.run_cli(cmd_env, set_cmd + f"--min {value + 2}")
        self.assertResultError(result, f"Rule create error: Rule may not be applied to {param1}")
        self.assertIn("Value is less than the minimum value", result.err())

        result = self.run_cli(cmd_env, set_cmd + f"--max {value - 2}")
        self.assertResultError(result, f"Rule create error: Rule may not be applied to {param1}")
        self.assertIn("Value is greater than the maximum value", result.err())

        #################
        result = self.run_cli(cmd_env, set_cmd + f"--min {value - 10}")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, set_cmd + f"--min {value + 3}")
        self.assertResultError(result, f"Rule update error: Rule may not be applied to {param1}")
        self.assertIn("Value is less than the minimum value", result.err())

        result = self.run_cli(cmd_env, set_cmd + f"--max {value + 10}")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, set_cmd + f"--max {value - 2}")
        self.assertResultError(result, f"Rule update error: Rule may not be applied to {param1}")
        self.assertIn("Value is greater than the maximum value", result.err())

        # bogus rules -- min/max out of whack
        err_msg = "Rule update error: Maximum constraint is less than an existing rule's minimum constraint"
        result = self.run_cli(cmd_env, set_cmd + f"--max {value - 11}")
        self.assertResultError(result, err_msg)
        err_msg = "Rule update error: Minimum constraint is greater than an existing rule's maximum constraint"
        result = self.run_cli(cmd_env, set_cmd + f"--min {value + 11}")
        self.assertResultError(result, err_msg)

        # delete the rules
        result = self.run_cli(cmd_env, set_cmd + "--no-min")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, set_cmd + "--no-max")
        self.assertResultSuccess(result)

        ################
        # negative tests for bad rule types: --max-len, --min-len, --regex
        result = self.run_cli(cmd_env, set_cmd + "--max-len -10 --min-len -1 --regex 'abc.*'")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max-len rules not valid for integer parameters", result.err())
        self.assertIn("min-len rules not valid for integer parameters", result.err())
        self.assertIn("regex rules not valid for integer parameters", result.err())

        result = self.run_cli(cmd_env, set_cmd + "--min-len 10")
        self.assertResultError(result, create_err_msg)
        self.assertIn("min-len rules not valid for integer parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},{value},{env_name},integer,0,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # see we don't leave any parameter behind when creating a parameter with an invalid rule
        self.delete_param(cmd_env, proj_name, param1)

        result = self.run_cli(cmd_env, set_cmd + "--type integer --value 9 --max-len 100")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max-len rules not valid for integer parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
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
        rules_cmd = param_cmd + "ls --rules -f csv -v"
        create_err_msg = "Rule create error"

        # create a basic parameter without a value, so the rule cannot be violated
        param1 = "param1"
        self.set_param(cmd_env, proj_name, param1, "true", param_type="boolean", env=env_name)
        self.unset_param(cmd_env, proj_name, param1, env=env_name)

        # see no rules
        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        set_cmd = param_cmd + f"set {param1} "

        ################
        # negative tests for bad rule types: --max, --min, --max-len, --min-len, --regex
        result = self.run_cli(cmd_env, set_cmd + "--max 100 --min 10 --max-len -10 --min-len -1 --regex 'abc.*'")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max rules not valid for boolean parameters", result.err())
        self.assertIn("min rules not valid for boolean parameters", result.err())
        self.assertIn("max-len rules not valid for boolean parameters", result.err())
        self.assertIn("min-len rules not valid for boolean parameters", result.err())
        self.assertIn("regex rules not valid for boolean parameters", result.err())

        result = self.run_cli(cmd_env, set_cmd + "--min-len 10")
        self.assertResultError(result, create_err_msg)
        self.assertIn("min-len rules not valid for boolean parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"{param1},-,,boolean,0,internal,false", result.out())

        result = self.run_cli(cmd_env, rules_cmd)
        self.assertResultSuccess(result)
        self.assertIn("No parameter rules found in project", result.out())

        #################
        # see we don't leave any parameter behind when creating a parameter with an invalid rule
        self.delete_param(cmd_env, proj_name, param1)

        result = self.run_cli(cmd_env, set_cmd + "--type boolean --value true --max 10")
        self.assertResultError(result, create_err_msg)
        self.assertIn("max rules not valid for boolean parameters", result.err())

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(self._empty_message(proj_name), result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

    def test_parameter_as_of_tag(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-param-tags")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("ptag-env")
        self.create_environment(cmd_env, env_name)

        param1 = "param1"
        value1a = "original"
        value1b = "updated"
        value1c = "final"

        self.set_param(cmd_env, proj_name, param1, value1a, env=env_name)
        details1a = self.get_param(cmd_env, proj_name, param1, env=env_name)

        self.set_param(cmd_env, proj_name, param1, value1b, env=env_name)
        details1b = self.get_param(cmd_env, proj_name, param1, env=env_name)

        # set tag here
        tag_name = "my-tag"  # scoped to an environment, so no need to 'make_name()'
        tag_set = base_cmd + f"env tag set '{env_name}' '{tag_name}' "
        result = self.run_cli(cmd_env, tag_set + "--desc 'quick tag'")
        self.assertResultSuccess(result)

        # set a value after the tag
        self.set_param(cmd_env, proj_name, param1, value1c, env=env_name)

        details = self.get_param(cmd_env, proj_name, param1, env=env_name, as_of=tag_name)
        self.assertEqual(details1b, details)

        details = self.get_param(cmd_env, proj_name, param1, env=env_name, as_of=details1a.get(PROP_MODIFIED))
        self.assertEqual(details1a, details)

        ################
        # error case where the timestamp is in the back
        timestamp = "2021-01-20"
        result = self.run_cli(cmd_env, tag_set + f"--time '{timestamp}'")
        self.assertResultSuccess(result)

        no_project = "No HistoricalProject matches the given query"
        param_cmd = base_cmd + f"--env '{env_name}' --project '{proj_name}' param "
        result = self.run_cli(cmd_env, param_cmd + f"ls -v --as-of '{tag_name}'")
        self.assertResultError(result, no_project)
        result = self.run_cli(cmd_env, param_cmd + f"diff --as-of '{tag_name}'")
        self.assertResultError(result, no_project)
        result = self.run_cli(cmd_env, param_cmd + f"env '{param1}' --as-of '{tag_name}'")
        self.assertResultError(result, no_project)
        result = self.run_cli(cmd_env, param_cmd + f"export docker --as-of '{tag_name}'")
        self.assertResultError(result, no_project)
        result = self.run_cli(cmd_env, param_cmd + f"get '{param1}' --as-of '{timestamp}'")
        self.assertResultError(result, no_project)

        ################
        # bad environment/tag combination testing
        bad_tag = "no-such-tag"
        tag_msg = f"Tag `{bad_tag}` could not be found in environment `{env_name}`"
        result = self.run_cli(cmd_env, param_cmd + f"ls -v --as-of {bad_tag}")
        self.assertResultError(result, tag_msg)
        result = self.run_cli(cmd_env, param_cmd + f"diff --as-of '{bad_tag}'")
        self.assertResultError(result, tag_msg)
        result = self.run_cli(cmd_env, param_cmd + f"env '{param1}' --as-of '{bad_tag}'")
        self.assertResultError(result, tag_msg)
        result = self.run_cli(cmd_env, param_cmd + f"export docker --as-of '{bad_tag}'")
        self.assertResultError(result, tag_msg)
        result = self.run_cli(cmd_env, param_cmd + f"get '{param1}' --as-of '{bad_tag}'")
        self.assertResultError(result, tag_msg)

        bad_env = "default"
        param_cmd = base_cmd + f"--project '{proj_name}' --env '{bad_env}' param "
        env_msg = f"Tag `{tag_name}` could not be found in environment `{bad_env}`"
        result = self.run_cli(cmd_env, param_cmd + f"ls -v --as-of {tag_name}")
        self.assertResultError(result, env_msg)
        result = self.run_cli(cmd_env, param_cmd + f"diff --as-of '{tag_name}'")
        self.assertResultError(result, env_msg)
        result = self.run_cli(cmd_env, param_cmd + f"env '{param1}' --as-of '{tag_name}'")
        self.assertResultError(result, env_msg)
        result = self.run_cli(cmd_env, param_cmd + f"export docker --as-of '{tag_name}'")
        self.assertResultError(result, env_msg)
        result = self.run_cli(cmd_env, param_cmd + f"get '{param1}' --as-of '{tag_name}'")
        self.assertResultError(result, env_msg)

        ################
        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

    def test_parameter_project_inheritance(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        # add a set of projects
        parent_name = self.make_name("test-param-proj-parent")
        child1_name = self.make_name("test-param-proj-child1")
        child2_name = self.make_name("test-param-proj-child2")

        self.create_project(cmd_env, parent_name)
        self.create_project(cmd_env, child1_name, parent=parent_name)
        self.create_project(cmd_env, child2_name, parent=parent_name)

        #########################
        # no --children parameters
        no_children_msg = "No parameters from a child project found in project "
        result = self.list_params(cmd_env, parent_name, show_children=True)
        self.assertIn(no_children_msg, result.out())
        result = self.list_params(cmd_env, child1_name, show_children=True)
        self.assertIn(no_children_msg, result.out())
        result = self.list_params(cmd_env, child2_name, show_children=True)
        self.assertIn(no_children_msg, result.out())

        #########################
        # setup a couple parameters on one of the children
        param1 = "param1"
        param2 = "secret2"

        # add some parameters to the one of the child projects
        value1 = "some_value"
        value2 = "ssshhhh"
        self.set_param(cmd_env, child1_name, param1, value1)
        self.set_param(cmd_env, child1_name, param2, value2, secret=True)

        #########################
        # normal 'param ls' testing -- parent and child2 have no values, but child1 does
        result = self.list_params(cmd_env, parent_name)
        self.assertIn(self._empty_message(parent_name), result.out())

        result = self.list_params(cmd_env, child1_name)
        self.assertIn(param1, result.out())
        self.assertIn(param2, result.out())

        result = self.list_params(cmd_env, child2_name)
        self.assertIn(self._empty_message(child2_name), result.out())

        #########################
        # no --parents parameters
        no_parent_msg = "No parameters from a parent project found in project "
        result = self.list_params(cmd_env, parent_name, show_parents=True)
        self.assertIn(no_parent_msg, result.out())
        result = self.list_params(cmd_env, child1_name, show_parents=True)
        self.assertIn(no_parent_msg, result.out())
        result = self.list_params(cmd_env, child2_name, show_parents=True)
        self.assertIn(no_parent_msg, result.out())

        #########################
        # see some children appear
        expected = f"""\
Name,Value,Project
{param1},{value1},{child1_name}
{param2},{REDACTED},{child1_name}
"""
        result = self.list_params(cmd_env, parent_name, show_children=True, fmt="csv")
        self.assertEqual(expected, result.out())
        result = self.list_params(cmd_env, child1_name, show_children=True, fmt="csv")
        self.assertIn(no_children_msg, result.out())
        result = self.list_params(cmd_env, child2_name, show_children=True, fmt="csv")
        self.assertIn(no_children_msg, result.out())

        #########################
        # setup parent with a couple variables
        param3 = "param3"
        param4 = "secret4"

        # add some parameters to the one of the child projects
        value3 = "some_value"
        value4 = "be vewy vewy quiet"
        self.set_param(cmd_env, parent_name, param3, value3)
        self.set_param(cmd_env, parent_name, param4, value4, secret=True)

        #########################
        # see parameters propagate correctly
        expected = f"""\
Name,Value,Project
{param3},{value3},{parent_name}
{param4},{REDACTED},{parent_name}
"""
        result = self.list_params(cmd_env, parent_name, show_parents=True, fmt="csv")
        self.assertIn(no_parent_msg, result.out())
        result = self.list_params(cmd_env, child1_name, show_parents=True, fmt="csv")
        self.assertEqual(expected, result.out())
        result = self.list_params(cmd_env, child2_name, show_parents=True, fmt="csv")
        self.assertEqual(expected, result.out())

        # again, with secrets
        expected = f"""\
Name,Value,Project
{param3},{value3},{parent_name}
{param4},{value4},{parent_name}
"""
        result = self.list_params(cmd_env, parent_name, show_parents=True, fmt="csv", secrets=True)
        self.assertIn(no_parent_msg, result.out())
        result = self.list_params(cmd_env, child1_name, show_parents=True, fmt="csv", secrets=True)
        self.assertEqual(expected, result.out())
        result = self.list_params(cmd_env, child2_name, show_parents=True, fmt="csv", secrets=True)
        self.assertEqual(expected, result.out())

        #########################
        # another level on project inheritance
        grandchild_name = self.make_name("test-param-proj-grand")
        self.create_project(cmd_env, grandchild_name, parent=child1_name)
        result = self.list_params(cmd_env, child1_name, show_parents=True, fmt="csv", secrets=True)
        self.assertEqual(expected, result.out())

        # a couple more parameters in the grandchild
        param5 = "param5"
        param6 = "secret6"
        value5 = "grand"
        value6 = "im hunting wabbits"
        self.set_param(cmd_env, grandchild_name, param5, value=value5)
        self.set_param(cmd_env, grandchild_name, param6, value=value6, secret=True)

        expected = f"""\
Name,Value,Project
{param1},{value1},{child1_name}
{param2},{REDACTED},{child1_name}
{param5},{value5},{grandchild_name}
{param6},{REDACTED},{grandchild_name}
"""
        result = self.list_params(cmd_env, parent_name, show_children=True, fmt="csv")
        self.assertEqual(expected, result.out())

        expected = f"""\
Name,Value,Project
{param5},{value5},{grandchild_name}
{param6},{REDACTED},{grandchild_name}
"""
        result = self.list_params(cmd_env, child1_name, show_children=True, fmt="csv")
        self.assertEqual(expected, result.out())

        #########################
        # attempt to delete parameters from the parent
        del_msg = f"Parameter '{param1}' must be deleted from project '{child1_name}'"
        del_cmd = base_cmd + f"--project '{grandchild_name}' param del -y '{param1}'"
        result = self.run_cli(cmd_env, del_cmd)
        self.assertResultError(result, del_msg)

        #########################
        # attempt to update parameters and values
        update_msg = f"Parameter '{param1}' must be set from project '{child1_name}'"
        set_cmd = base_cmd + f"--project '{grandchild_name}' param set '{param1}' -d 'new desc'"
        result = self.run_cli(cmd_env, set_cmd)
        self.assertResultError(result, update_msg)

        set_cmd = base_cmd + f"--project '{grandchild_name}' param set '{param1}' -v 'next value'"
        result = self.run_cli(cmd_env, set_cmd)
        self.assertResultError(result, update_msg)

        #########################
        # add parameters with same names, but in different children of the parent
        value5a = "slam"
        value6a = "kill the wabbit"
        self.set_param(cmd_env, child2_name, param5, value=value5a)
        self.set_param(cmd_env, child2_name, param6, value=value6a)  # NOTE: not a secret!

        expected = f"""\
Name,Value,Project
{param1},{value1},{child1_name}
{param2},{REDACTED},{child1_name}
{param5},{value5},{grandchild_name}
{param6},{REDACTED},{grandchild_name}
{param5},{value5a},{child2_name}
{param6},{value6a},{child2_name}
"""
        result = self.list_params(cmd_env, parent_name, show_children=True, fmt="csv")
        self.assertEqual(expected, result.out())

        # cleanup
        self.delete_project(cmd_env, grandchild_name)
        self.delete_project(cmd_env, child1_name)
        self.delete_project(cmd_env, child2_name)
        self.delete_project(cmd_env, parent_name)

    def test_parameter_pagination(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        # add a project
        proj_name = self.make_name("test-param-pagination")
        self.create_project(cmd_env, proj_name)

        param_base = "param"
        param_count = TEST_PAGE_SIZE + 1
        for idx in range(param_count):
            self.set_param(cmd_env, proj_name, f"{param_base}{idx}", )

        list_cmd = base_cmd + f"--project {proj_name} param ls"
        self.assertPaginated(cmd_env, list_cmd, "/parameters/?")

        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        output = result.out()
        for idx in range(param_count):
            self.assertIn(f"{param_base}{idx}", output)

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_parameter_over_specified(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        # add a project
        proj_name = self.make_name("test-param-overdone")
        self.create_project(cmd_env, proj_name)

        filename = self.make_name("cooked")
        self.write_file(filename, "bogus value from file")

        err_msg = "Conflicting arguments: cannot specify more than one"
        set_cmd = base_cmd + f"--project {proj_name} param set param1 "
        values = [
            f"-i {filename} ",
            "-v value ",
            "--prompt ",
            "--generate ",
            "--fqn github://cloudtruth/cloudtruth-cli/main/README.md "
        ]
        for index, first in enumerate(values):
            if index + 1 == len(values):
                break
            for second in values[index + 1:]:
                result = self.run_cli(cmd_env, set_cmd + first + second)
                self.assertResultError(result, err_msg)

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)

    def test_parameter_generate(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()
        default_len = 12

        # add a project
        proj_name = self.make_name("test-param-overdone")
        self.create_project(cmd_env, proj_name)

        param_cmd = base_cmd + f"--project {proj_name} param "
        list_cmd = param_cmd + "ls -sf json"
        param1 = "param1"
        param2 = "param2"

        result1 = self.run_cli(cmd_env, param_cmd + f"set {param1} --generate")
        self.assertResultSuccess(result1)
        self.assertIn(f"Set parameter '{param1}'", result1.out())
        result2 = self.run_cli(cmd_env, param_cmd + f"set {param2} --generate --secret true")
        self.assertResultSuccess(result2)
        self.assertIn(f"Set parameter '{param2}'", result2.out())

        # see secrets are secret, but generated values do not need to be secret
        entries = self.get_cli_entries(cmd_env, list_cmd, "parameter")
        entry = find_by_prop(entries, PROP_NAME, param1)[0]
        value1 = entry.get(PROP_VALUE)
        self.assertEqual(len(value1), default_len)
        self.assertEqual(entry.get("Secret"), "false")
        entry = find_by_prop(entries, PROP_NAME, param2)[0]
        value2 = entry.get(PROP_VALUE)
        self.assertEqual(len(value2), default_len)
        self.assertEqual(entry.get("Secret"), "true")

        # check that values were not shown during generation
        self.assertNotIn(value1, result1.out())
        self.assertNotIn(value2, result2.out())
        self.assertNotEqual(value1, value2)

        # update the values
        result1 = self.run_cli(cmd_env, param_cmd + f"set {param1} --generate")
        self.assertResultSuccess(result1)
        self.assertIn(f"Updated parameter '{param1}'", result1.out())
        result2 = self.run_cli(cmd_env, param_cmd + f"set {param2} --generate")
        self.assertResultSuccess(result2)
        self.assertIn(f"Updated parameter '{param2}'", result2.out())

        entries = self.get_cli_entries(cmd_env, list_cmd, "parameter")
        entry = find_by_prop(entries, PROP_NAME, param1)[0]
        value1a = entry.get(PROP_VALUE)
        self.assertEqual(len(value1a), default_len)
        self.assertEqual(entry.get("Secret"), "false")
        entry = find_by_prop(entries, PROP_NAME, param2)[0]
        value2a = entry.get(PROP_VALUE)
        self.assertEqual(len(value2a), default_len)
        self.assertEqual(entry.get("Secret"), "true")

        # check to make sure values were updated
        self.assertNotEqual(value1, value1a)
        self.assertNotEqual(value2, value2a)

        ##########################
        # does not work with boolean/integer types
        param3 = "param3"
        param4 = "param4"
        value3 = "true"
        value4 = "123456"
        self.set_param(cmd_env, proj_name, param3, value=value3, param_type="boolean")
        self.set_param(cmd_env, proj_name, param4, value=value4, param_type="integer")

        err_msg = "Rule violation: Value is not of type "
        result = self.run_cli(cmd_env, param_cmd + f"set {param3} --generate")
        self.assertResultError(result, err_msg + "boolean")
        result = self.run_cli(cmd_env, param_cmd + f"set {param4} --generate")
        self.assertResultError(result, err_msg + "integer")

        ##########################
        # does not work with rules... should possibly change next iteration
        param5 = "param5"
        min_len = 50
        result = self.run_cli(cmd_env, param_cmd + f"set {param5} --min-len {min_len} --generate")
        self.assertResultError(result, f"Rule violation: Value must be at least {min_len} characters")

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_parameter_drift(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        PROP_SHELL = "Shell"
        PROP_SERVER = "CloudTruth"
        PROP_CHANGE = "Difference"

        # add a project with some variables
        proj_name = self.make_name("test-param-drift")
        param1 = "param1"
        value1 = "my-param-value"
        param2 = "param2"
        value2 = "ssssshhhhh"
        param3 = "param3"
        value3 = "another-param-value"
        env3 = "different in shell"
        param4 = "param4"
        value4 = "be vewwwwy qwiet"
        env4 = "im hunting wabbits"
        param5 = "param5"
        value5 = "vanilla"
        param6 = "param6"
        value6 = "ssssshhhhhh"
        param7 = "param7"
        env7 = "env-value"
        empty = ""
        self.create_project(cmd_env, proj_name)
        self.set_param(cmd_env, proj_name, param1, value=value1)
        self.set_param(cmd_env, proj_name, param2, value=value2, secret=True)
        self.set_param(cmd_env, proj_name, param3, value=value3)
        self.set_param(cmd_env, proj_name, param4, value=value4, secret=True)
        self.set_param(cmd_env, proj_name, param5, value=value5)
        self.set_param(cmd_env, proj_name, param6, value=value6, secret=True)

        # setup parameters in the shell environment
        cmd_env["CLOUDTRUTH_PROJECT"] = proj_name
        cmd_env[param1] = value1
        cmd_env[param2] = value2
        cmd_env[param3] = env3
        cmd_env[param4] = env4
        cmd_env[param7] = env7

        drift_cmd = base_cmd + "param drift "

        # just the names
        result = self.run_cli(cmd_env, drift_cmd)
        self.assertResultSuccess(result)
        self.assertNotIn(param1, result.stdout)
        self.assertNotIn(param2, result.stdout)
        self.assertIn(param3, result.stdout)
        self.assertIn(param4, result.stdout)
        self.assertIn(param5, result.stdout)
        self.assertIn(param6, result.stdout)
        self.assertIn(param7, result.stdout)
        # skip some standard names
        self.assertNotIn("HOME", result.stdout)
        self.assertNotIn("PWD", result.stdout)
        self.assertNotIn("CLOUDTRUTH_PROFILE", result.stdout)
        self.assertNotIn("CLOUDTRUTH_PROJECT", result.stdout)
        self.assertNotIn("CLOUDTRUTH_ENVIRONMENT", result.stdout)
        self.assertNotIn("CLOUDTRUTH_API_KEY", result.stdout)

        # values without secrets shown
        entries = self.get_cli_entries(cmd_env, drift_cmd + "-f json", "parameter-drift")
        self.assertEqual(len(find_by_prop(entries, PROP_NAME, param1)), 0)
        self.assertEqual(len(find_by_prop(entries, PROP_NAME, param2)), 0)
        entry = find_by_prop(entries, PROP_NAME, param3)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "changed")
        self.assertEqual(entry.get(PROP_SHELL), env3)
        self.assertEqual(entry.get(PROP_SERVER), value3)
        entry = find_by_prop(entries, PROP_NAME, param4)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "changed")
        self.assertEqual(entry.get(PROP_SHELL), REDACTED)
        self.assertEqual(entry.get(PROP_SERVER), REDACTED)
        entry = find_by_prop(entries, PROP_NAME, param5)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "removed")
        self.assertEqual(entry.get(PROP_SHELL), empty)
        self.assertEqual(entry.get(PROP_SERVER), value5)
        entry = find_by_prop(entries, PROP_NAME, param6)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "removed")
        self.assertEqual(entry.get(PROP_SHELL), empty)
        self.assertEqual(entry.get(PROP_SERVER), REDACTED)
        entry = find_by_prop(entries, PROP_NAME, param7)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "added")
        self.assertEqual(entry.get(PROP_SHELL), env7)
        self.assertEqual(entry.get(PROP_SERVER), empty)

        # values with secrets shown
        entries = self.get_cli_entries(cmd_env, drift_cmd + "-sf json", "parameter-drift")
        self.assertEqual(len(find_by_prop(entries, PROP_NAME, param1)), 0)
        self.assertEqual(len(find_by_prop(entries, PROP_NAME, param2)), 0)
        entry = find_by_prop(entries, PROP_NAME, param3)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "changed")
        self.assertEqual(entry.get(PROP_SHELL), env3)
        self.assertEqual(entry.get(PROP_SERVER), value3)
        entry = find_by_prop(entries, PROP_NAME, param4)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "changed")
        self.assertEqual(entry.get(PROP_SHELL), env4)
        self.assertEqual(entry.get(PROP_SERVER), value4)
        entry = find_by_prop(entries, PROP_NAME, param5)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "removed")
        self.assertEqual(entry.get(PROP_SHELL), empty)
        self.assertEqual(entry.get(PROP_SERVER), value5)
        entry = find_by_prop(entries, PROP_NAME, param6)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "removed")
        self.assertEqual(entry.get(PROP_SHELL), empty)
        self.assertEqual(entry.get(PROP_SERVER), value6)
        entry = find_by_prop(entries, PROP_NAME, param7)[0]
        self.assertEqual(entry.get(PROP_CHANGE), "added")
        self.assertEqual(entry.get(PROP_SHELL), env7)
        self.assertEqual(entry.get(PROP_SERVER), empty)

        # combination test... 'cloudtruth run' should put all parameters/values in the shell environment, so all the
        # drift is standard OS/shell parameters that are added
        run_cmd = base_cmd + f"run -c '{base_cmd} param drift -vf json'"
        entries = self.get_cli_entries(cmd_env, run_cmd, "parameter-drift")
        self.assertEqual(len(entries), len(find_by_prop(entries, PROP_CHANGE, "added")))

        # cleanup
        self.delete_project(cmd_env, proj_name)

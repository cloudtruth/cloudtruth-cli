import os

from testcase import TestCase, DEFAULT_ENV_NAME, REDACTED, DEFAULT_PARAM_VALUE


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
+----------+-------------+---------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Type   | Secret | Description                     |
+----------+-------------+---------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | static | false  | this is just a test description |
+----------+-------------+---------+--------+--------+---------------------------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Type,Secret,Description
my_param,cRaZy value,default,static,false,this is just a test description
""")
        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{value1}", result.out())

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
        self.assertIn("Please provide at least one of", result.err())

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
+----------+-------+---------+--------+--------+-----------------+
| Name     | Value | Source  | Type   | Secret | Description     |
+----------+-------+---------+--------+--------+-----------------+
| my_param | ***** | default | static | true   | my secret value |
+----------+-------+---------+--------+--------+-----------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), f"""\
Name,Value,Source,Type,Secret,Description
my_param,{REDACTED},default,static,true,my secret value
""")

        # now, display with the secrets value
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
+----------+-----------------------+---------+--------+--------+-----------------+
| Name     | Value                 | Source  | Type   | Secret | Description     |
+----------+-----------------------+---------+--------+--------+-----------------+
| my_param | super-SENSITIVE-vAluE | default | static | true   | my secret value |
+----------+-----------------------+---------+--------+--------+-----------------+
""")

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets --format csv")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Type,Secret,Description
my_param,super-SENSITIVE-vAluE,default,static,true,my secret value
""")

        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{value1}", result.out())

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
+-----------+------------+---------+--------+--------+-------------+
| Name      | Value      | Source  | Type   | Secret | Description |
+-----------+------------+---------+--------+--------+-------------+
| sensitive | classified | default | static | true   |             |
| sna       | foo        | default | static | false  |             |
+-----------+------------+---------+--------+--------+-------------+
""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param ls -v -s")
        self.assertEqual(result.out(), """\
+-----------+------------+---------+--------+--------+-------------+
| Name      | Value      | Source  | Type   | Secret | Description |
+-----------+------------+---------+--------+--------+-------------+
| sensitive | top-secret | default | static | true   |             |
| sna       | fu         | default | static | false  |             |
+-----------+------------+---------+--------+--------+-------------+
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
+----------+-------------+---------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Type   | Secret | Description                     |
+----------+-------------+---------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | static | false  | this is just a test description |
+----------+-------------+---------+--------+--------+---------------------------------+
""")

        # switch it to a secret
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret true")
        self.assertEqual(result.return_value, 0)

        # see that it has been changed to a secret (redacted in cli)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------+---------+--------+--------+---------------------------------+
| Name     | Value | Source  | Type   | Secret | Description                     |
+----------+-------+---------+--------+--------+---------------------------------+
| my_param | ***** | default | static | true   | this is just a test description |
+----------+-------+---------+--------+--------+---------------------------------+
""")

        # verify value has not changed
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Type   | Secret | Description                     |
+----------+-------------+---------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | static | true   | this is just a test description |
+----------+-------------+---------+--------+--------+---------------------------------+
""")

        # switch back to a regular parameter
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret false")
        self.assertEqual(result.return_value, 0)

        # see that it is no longer redacted
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+--------+--------+---------------------------------+
| Name     | Value       | Source  | Type   | Secret | Description                     |
+----------+-------------+---------+--------+--------+---------------------------------+
| my_param | cRaZy value | default | static | false  | this is just a test description |
+----------+-------------+---------+--------+--------+---------------------------------+
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
+----------+----------------------+---------+--------+--------+---------------------------+
| Name     | Value                | Source  | Type   | Secret | Description               |
+----------+----------------------+---------+--------+--------+---------------------------+
| my_param | static val from file | default | static | false  | param set from file input |
+----------+----------------------+---------+--------+--------+---------------------------+
""")

        # change value from `--value` flag from CLI
        value2 = "update-from-value"
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --value '{value2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------------+---------+--------+--------+---------------------------+
| Name     | Value             | Source  | Type   | Secret | Description               |
+----------+-------------------+---------+--------+--------+---------------------------+
| my_param | update-from-value | default | static | false  | param set from file input |
+----------+-------------------+---------+--------+--------+---------------------------+
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
+----------+---------------------+---------+--------+--------+---------------------------+
| Name     | Value               | Source  | Type   | Secret | Description               |
+----------+---------------------+---------+--------+--------+---------------------------+
| my_param | another-static-file | default | static | false  | param set from file input |
+----------+---------------------+---------+--------+--------+---------------------------+
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
        # no such FQN
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())

        # again, with a JMES path
        result = self.run_cli(cmd_env, sub_cmd + f"set '{key1}' --fqn '{fqn}' --jmes '{jmes}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(invalid_fqn_msg, result.err())

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
+-----------+--------------------------------+---------+--------+--------+-------------+
| Name      | Value                          | Source  | Type   | Secret | Description |
+-----------+--------------------------------+---------+--------+--------+-------------+
| speicla14 | *****                          | default | static | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | static | false  | Jade lunch  |
+-----------+--------------------------------+---------+--------+--------+-------------+
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -s")
        self.assertEqual(result.out(), """\
+-----------+--------------------------------+---------+--------+--------+-------------+
| Name      | Value                          | Source  | Type   | Secret | Description |
+-----------+--------------------------------+---------+--------+--------+-------------+
| speicla14 | cueey-chicken                  | default | static | true   | Jade secret |
| speicla3  | beef brocolli, pork fried rice | default | static | false  | Jade lunch  |
+-----------+--------------------------------+---------+--------+--------+-------------+
""")

        #################
        # CSV format
        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertEqual(result.out(), f"""\
Name,Value,Source,Type,Secret,Description
speicla14,{REDACTED},default,static,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,static,false,Jade lunch
""")

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv -s")
        self.assertEqual(result.out(), """\
Name,Value,Source,Type,Secret,Description
speicla14,cueey-chicken,default,static,true,Jade secret
speicla3,"beef brocolli, pork fried rice",default,static,false,Jade lunch
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
      "Secret": "true",
      "Source": "default",
      "Type": "static",
      "Value": "*****"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
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
      "Secret": "true",
      "Source": "default",
      "Type": "static",
      "Value": "cueey-chicken"
    },
    {
      "Description": "Jade lunch",
      "Name": "speicla3",
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
    Secret: "true"
    Source: default
    Type: static
    Value: "{REDACTED}"
  - Description: Jade lunch
    Name: speicla3
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
    Secret: "true"
    Source: default
    Type: static
    Value: cueey-chicken
  - Description: Jade lunch
    Name: speicla3
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
            self.verify_param(cmd_env, proj_name, param_name, param_value, secret=False)

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
        diff_cmd = sub_cmd + f"diff '{env_a}' '{env_b}' -f csv "
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

        # when specifying properties where there are no diffs, we get nothing
        result = self.run_cli(cmd_env, diff_cmd + "-s --property fqn")
        self.assertIn("No parameters or differences in compared properties found", result.out())

        #####################
        # Error cases

        # no comparing to yourself
        result = self.run_cli(cmd_env, sub_cmd + f"difference '{env_a}' '{env_a}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn("Invalid comparing an environment to itself", result.err())

        # first environment DNE
        result = self.run_cli(cmd_env, sub_cmd + "differ 'charlie-foxtrot' '{env_b}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Did not find environment 'charlie-foxtrot'", result.err())

        # second environment DNE
        result = self.run_cli(cmd_env, sub_cmd + f"differences '{env_a}' 'missing'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn("Did not find environment 'missing'", result.err())

        # cleanup
        self.delete_environment(cmd_env, env_a)
        self.delete_environment(cmd_env, env_b)
        self.delete_project(cmd_env, proj_name)

from testcase import TestCase, DEFAULT_ENV_NAME


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

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v -f csv")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Description
my_param,cRaZy value,default,this is just a test description
""")
        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{value1}", result.out())

        # idempotent -- same value
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value1, result.out())

        ########
        # update the just the value
        value2 = "new_value"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # update the just the description
        desc2 = "alt description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} -d '{desc2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc2))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # idempotent
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Failed to remove parameter '{key1}'", result.out())

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
        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v -f csv")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Description
my_param,*****,default,my secret value
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

        # use CSV
        result = self.run_cli(cmd_env, sub_cmd + f"list --values --secrets --format csv")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))
        self.assertEqual(result.out(), """\
Name,Value,Source,Description
my_param,super-SENSITIVE-vAluE,default,my secret value
""")

        # get the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"{value1}", result.out())

        # idempotent -- same value
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value1}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v -s")
        self.assertTrue(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))

        # make sure it is still a secret
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertFalse(result.out_contains_both(key1, value1))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value1, result.out())

        ########
        # update the just the value
        value2 = "new_value"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --value '{value2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v -s")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc1))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # update the just the description
        desc2 = "alt description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} -d '{desc2}'")
        self.assertEqual(result.return_value, 0)

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v -s")
        self.assertTrue(result.out_contains_both(key1, value2))
        self.assertTrue(result.out_contains_both(key1, desc2))

        result = self.run_cli(cmd_env, sub_cmd + f"get {key1}")
        self.assertIn(value2, result.out())

        ########
        # delete the parameter
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)

        # make sure it is gone
        result = self.run_cli(cmd_env, sub_cmd + "list --values --secrets")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(empty_msg))

        # idempotent
        result = self.run_cli(cmd_env, sub_cmd + f"delete {key1}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Failed to remove parameter '{key1}'", result.out())

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

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name1} param export docker")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
SNA=foo

""")

        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name2} param export docker")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
SNA=fu

""")

        self.delete_project(cmd_env, proj_name1)
        self.delete_project(cmd_env, proj_name2)

    def test_environment_separation(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        proj_name = "baseball" # self.make_name("test-prj-env-sep")
        self.create_project(cmd_env, proj_name)

        env_name1 = DEFAULT_ENV_NAME  # no job-id variation
        env_name2 = "test-mets" # self.make_name("test-env-foo")
        self.create_environment(cmd_env, env_name2)
        env_name3 = "test-redsox" # self.make_name("test-env-bar")
        self.create_environment(cmd_env, env_name3, parent=env_name2)

        var1_name = "base"
        var1_value1 = "first"
        var1_value2 = "second"
        var1_value3 = "thrid"
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
        self.assertEqual(result.out(), """\
SECOND_PARAM=a value with spaces
FIRST_PARAM=posix_compliant_value

""")

        result = self.run_cli(cmd_env, docker_cmd + "--secrets")
        self.assertEqual(result.out(), """\
FIRST_PARAM_SECRET=top-secret-sci
SECOND_PARAM=a value with spaces
FIRST_PARAM=posix_compliant_value
SECOND_SECRET=sensitive value with spaces

""")

        result = self.run_cli(cmd_env, docker_cmd + "--secrets --starts-with SECOND")
        self.assertEqual(result.out(), """\
SECOND_PARAM=a value with spaces

""")

        # use uppercase key without secrets
        result = self.run_cli(cmd_env, docker_cmd + "--starts-with FIRST")
        self.assertEqual(result.out(), """\

""")

        # use uppercase key with secrets
        result = self.run_cli(cmd_env, docker_cmd + "--starts-with FIRST -s")
        self.assertEqual(result.out(), """\
FIRST_PARAM_SECRET=top-secret-sci

""")

        # use lowercase key with secrets
        result = self.run_cli(cmd_env, docker_cmd + "--contains param -s")
        self.assertEqual(result.out(), """\
FIRST_PARAM=posix_compliant_value

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
        self.assertEqual(result.out(), """\
SECOND_PARAM="a value with spaces"
FIRST_PARAM="posix_compliant_value"

""")

        #####################
        # Shell
        shell_cmd = base_cmd + f"--project {proj_name} param export shell "
        result = self.run_cli(cmd_env, shell_cmd)
        self.assertEqual(result.return_value, 0)
        self.assertEqual(result.out(), """\
SECOND_PARAM=a\ value\ with\ spaces
FIRST_PARAM=posix_compliant_value

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

        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+---------------------------------+
| Name     | Value       | Source  | Description                     |
+----------+-------------+---------+---------------------------------+
| my_param | cRaZy value | default | this is just a test description |
+----------+-------------+---------+---------------------------------+
""")

        # switch it to a secret
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret true")
        self.assertEqual(result.return_value, 0)

        # see that it has been changed to a secret (redacted in cli)
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertEqual(result.out(), """\
+----------+-------+---------+---------------------------------+
| Name     | Value | Source  | Description                     |
+----------+-------+---------+---------------------------------+
| my_param | ***** | default | this is just a test description |
+----------+-------+---------+---------------------------------+
""")

        # verify value has not changed
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v -s")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+---------------------------------+
| Name     | Value       | Source  | Description                     |
+----------+-------------+---------+---------------------------------+
| my_param | cRaZy value | default | this is just a test description |
+----------+-------------+---------+---------------------------------+
""")

        # switch back to a regular parameter
        result = self.run_cli(cmd_env, sub_cmd + f"set {key1} --secret false")
        self.assertEqual(result.return_value, 0)

        # see that it is no longer redacted
        result = self.run_cli(cmd_env, sub_cmd + f"ls -v")
        self.assertEqual(result.out(), """\
+----------+-------------+---------+---------------------------------+
| Name     | Value       | Source  | Description                     |
+----------+-------------+---------+---------------------------------+
| my_param | cRaZy value | default | this is just a test description |
+----------+-------------+---------+---------------------------------+
""")

        self.delete_project(cmd_env, proj_name)

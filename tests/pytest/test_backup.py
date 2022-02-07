from testcase import TestCase


class TestProjects(TestCase):
    def test_backup_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        backup_cmd = base_cmd + "backup "
        snap_cmd = backup_cmd + "snapshot -y "

        type1 = self.make_name("test-back-int")
        type2 = self.make_name("test-back-str")
        type1_max = "4096"
        type1_min = "-511"
        env_a = self.make_name("test-back-env_a")
        env_b = self.make_name("test-back-env_b")
        proj1 = self.make_name("test-back-proj1")
        proj2 = self.make_name("test-back-proj2")
        p2_max_len = "100"
        p2_min_len = "10"
        temp1 = "my_temp"
        param1 = "my_param"
        param2 = "another_param"
        value1a = "1234"
        value1b = "-42"
        value2a = "sssssshhhhhh"
        value2b = "be veeeewy qwiet"
        body1 = f"This refers to {{{{ {param1} }}}}"

        # create types with a couple rules
        self.create_type(cmd_env, type1, parent="integer", extra=f"--max {type1_max} --min {type1_min}")
        self.create_type(cmd_env, type2)

        self.create_environment(cmd_env, env_a)
        self.create_environment(cmd_env, env_b, parent=env_a)

        self.create_project(cmd_env, proj1)
        self.create_project(cmd_env, proj2, parent=proj1)

        secret = True
        self.set_param(cmd_env, proj1, param1, env=env_a, value=value1a, param_type=type1)
        self.set_param(cmd_env, proj1, param1, env=env_b, value=value1b)
        extra = f"--min-len {p2_min_len} --max-len {p2_max_len}"
        self.set_param(cmd_env, proj2, param2, env=env_a, value=value2a, secret=secret, extra=extra)
        self.set_param(cmd_env, proj2, param2, env=env_b, value=value2b)

        self.set_template(cmd_env, proj1, temp1, body1)

        result = self.run_cli(cmd_env, snap_cmd + "-f json")
        self.assertResultSuccess(result)
        # massaging to avoid using a real parser (and avoid issues on Windows)
        modified = result.out()
        modified = modified.replace(": null", ": None")
        modified = modified.replace(": true", ": True")
        modified = modified.replace(": false", ": False")
        snapshot = eval(modified)

        #############################
        # types validation
        cttypes = snapshot.get("types")
        ct_type = cttypes.get(type1)
        self.assertEqual(ct_type.get("name"), type1)
        self.assertEqual(ct_type.get("parent"), "integer")
        rules = ct_type.get("rules")
        self.assertEqual(sorted(list(rules.keys())), ["max", "min"])
        rule = rules.get("max")
        self.assertEqual(rule.get("rule_type"), "max")
        self.assertEqual(rule.get("constraint"), type1_max)
        rule = rules.get("min")
        self.assertEqual(rule.get("rule_type"), "min")
        self.assertEqual(rule.get("constraint"), type1_min)

        ct_type = cttypes.get(type2)
        self.assertEqual(ct_type.get("name"), type2)
        self.assertEqual(ct_type.get("parent"), "string")
        rules = ct_type.get("rules")
        self.assertEqual(list(rules.keys()), [])

        #############################
        # environment validation
        environments = snapshot.get("environments")
        env = environments.get(env_a)
        self.assertEqual(env.get("name"), env_a)
        self.assertIsNotNone(env.get("parent"))
        env = environments.get(env_b)
        self.assertEqual(env.get("name"), env_b)
        self.assertEqual(env.get("parent"), env_a)

        #############################
        # project validation
        projects = snapshot.get("projects")

        proj = projects.get(proj1)
        self.assertEqual(proj.get("name"), proj1)
        self.assertEqual(proj.get("parent"), None)

        templates = proj.get("templates")
        self.assertEqual(sorted(list(templates.keys())), [temp1])
        temp = templates.get(temp1)
        self.assertEqual(temp.get("name"), temp1)
        self.assertEqual(temp.get("text"), body1)

        parameters = proj.get("parameters")
        self.assertEqual(list(parameters.keys()), [param1])
        param = parameters.get(param1)
        self.assertEqual(param.get("name"), param1)
        self.assertEqual(param.get("secret"), False)
        self.assertEqual(param.get("param_type"), type1)
        self.assertEqual(param.get("project"), proj1)
        rules = param.get("rules")
        self.assertEqual(sorted(list(rules.keys())), [])
        values = param.get("values")
        self.assertEqual(sorted(list(values.keys())), [env_a, env_b])
        v = values.get(env_a)
        self.assertEqual(v.get("environment"), env_a)
        self.assertEqual(v.get("source"), env_a)
        self.assertEqual(v.get("external"), None)
        self.assertEqual(v.get("evaluated"), False)
        self.assertEqual(v.get("value"), value1a)
        v = values.get(env_b)
        self.assertEqual(v.get("environment"), env_b)
        self.assertEqual(v.get("source"), env_b)
        self.assertEqual(v.get("external"), None)
        self.assertEqual(v.get("evaluated"), False)
        self.assertEqual(v.get("value"), value1b)

        proj = projects.get(proj2)
        self.assertEqual(proj.get("name"), proj2)
        self.assertEqual(proj.get("parent"), proj1)

        templates = proj.get("templates")
        self.assertEqual(sorted(list(templates.keys())), [])

        parameters = proj.get("parameters")
        self.assertEqual(list(parameters.keys()), [param2])
        param = parameters.get(param2)
        self.assertEqual(param.get("name"), param2)
        self.assertEqual(param.get("secret"), secret)
        self.assertEqual(param.get("param_type"), "string")
        self.assertEqual(param.get("project"), proj2)
        rules = param.get("rules")
        self.assertEqual(sorted(list(rules.keys())), ["max_len", "min_len"])
        rule = rules.get("max_len")
        self.assertEqual(rule.get("rule_type"), "max_len")
        self.assertEqual(rule.get("constraint"), p2_max_len)
        rule = rules.get("min_len")
        self.assertEqual(rule.get("rule_type"), "min_len")
        self.assertEqual(rule.get("constraint"), p2_min_len)
        values = param.get("values")
        self.assertEqual(sorted(list(values.keys())), [env_a, env_b])
        v = values.get(env_a)
        self.assertEqual(v.get("environment"), env_a)
        self.assertEqual(v.get("source"), env_a)
        self.assertEqual(v.get("external"), None)
        self.assertEqual(v.get("evaluated"), False)
        self.assertEqual(v.get("value"), value2a)
        v = values.get(env_b)
        self.assertEqual(v.get("environment"), env_b)
        self.assertEqual(v.get("source"), env_b)
        self.assertEqual(v.get("external"), None)
        self.assertEqual(v.get("evaluated"), False)
        self.assertEqual(v.get("value"), value2b)

        #############################
        # yaml and default (yaml) outputs
        for extra in ["", "-f yaml"]:
            result = self.run_cli(cmd_env, snap_cmd + extra)
            self.assertResultSuccess(result)
            text = result.out()
            self.assertIn(f"{type1}:", text)
            self.assertIn(f"{type2}:", text)
            self.assertIn(f"constraint: \"{type1_min}\"", text)
            self.assertIn(f"constraint: \"{type1_max}\"", text)

            self.assertIn(f"{env_a}:", text)
            self.assertIn(f"{env_b}:", text)
            self.assertIn(f"parent: {env_a}", text)

            self.assertIn(f"{proj1}:", text)
            self.assertIn(f"{proj2}:", text)

        # cleanup
        self.delete_project(cmd_env, proj2)
        self.delete_project(cmd_env, proj1)
        self.delete_environment(cmd_env, env_b)
        self.delete_environment(cmd_env, env_a)
        self.delete_type(cmd_env, type2)
        self.delete_type(cmd_env, type1)

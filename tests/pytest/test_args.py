"""
Tests precedence of command line arguments, profiles(?), and environment variables.
"""
from testcase import TestCase
from testcase import CT_ENV, CT_PROFILE, CT_PROJ


class TestTopLevelArgs(TestCase):

    def test_arg_priority(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        printenv = f" run -i none -- {self.get_display_env_command()}"
        proj1 = self.make_name("test-arg-project-1")
        proj2 = self.make_name("test-arg-proj2")
        env1 = self.make_name("dev a")
        env2 = self.make_name("dev B")

        self.create_project(cmd_env, proj1)
        self.create_project(cmd_env, proj2)
        self.create_environment(cmd_env, env1)
        self.create_environment(cmd_env, env2)

        # remote things to make sure we have a "clean" environment
        cmd_env.pop(CT_PROJ, 'No project')
        cmd_env.pop(CT_ENV, 'No environment')

        # the CLOUDTRUTH_PROFILE cannot be removed, since it may change the server-url/api-key, but
        # need to accommodate for the profile also setting the project/environment variables
        def_proj = "default"
        def_env = "default"
        prof_name = cmd_env.get(CT_PROFILE, None)
        if prof_name:
            profile = self.get_profile(cmd_env, prof_name)
            if profile:
                def_proj = profile.get("Project", None) or "default"
                def_env = profile.get("Environment", None) or "default"

        # check defaults are used
        result = self.run_cli(cmd_env, base_cmd + printenv)
        self.assertIn(f"{CT_PROJ}={def_proj}", result.out())
        self.assertIn(f"{CT_ENV}={def_env}", result.out())

        # set project/environment in environment
        cmd_env[CT_PROJ] = proj1
        cmd_env[CT_ENV] = env1

        # see items picked up from environment
        result = self.run_cli(cmd_env, base_cmd + printenv)
        self.assertIn(f"{CT_PROJ}={proj1}", result.out())
        self.assertIn(f"{CT_ENV}={env1}", result.out())

        # see that CLI arguments override the environment
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj2}' --env '{env2}'" + printenv)
        self.assertIn(f"{CT_PROJ}={proj2}", result.out())
        self.assertIn(f"{CT_ENV}={env2}", result.out())

        # mix and match
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj2}'" + printenv)
        self.assertIn(f"{CT_PROJ}={proj2}", result.out())
        self.assertIn(f"{CT_ENV}={env1}", result.out())

        result = self.run_cli(cmd_env, base_cmd + f"--env '{env2}'" + printenv)
        self.assertIn(f"{CT_PROJ}={proj1}", result.out())
        self.assertIn(f"{CT_ENV}={env2}", result.out())

        # cleanup
        self.delete_project(cmd_env, proj1)
        self.delete_project(cmd_env, proj2)
        self.delete_environment(cmd_env, env1)
        self.delete_environment(cmd_env, env2)

    def test_arg_missing_subcommand(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        for (subcmd, aliases) in {
            "config": ["configuration"],
            "environments": ["environment", "envs", "env", "e"],
            "integrations": ["integration", "integrate", "int"],
            "parameters": ["parameter", "params", "param", "p"],
            "projects": ["project", "proj"],
            "run": ["r"],
            "templates": ["template", "t"],
        }.items():
            for alias in [subcmd] + aliases:
                result = self.run_cli(cmd_env, base_cmd + alias)
                self.assertEqual(result.return_value, 0)
                self.assertIn(f"No '{subcmd}' sub-command executed", result.err())

    def test_arg_resolution(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-unknown-proj")
        env_name = self.make_name("test-env-unknown")
        checked_commands = [
            "param ls -v",
            "templates ls -v",
            f"run -i none -c {self.get_display_env_command()}",
        ]
        unchecked_commands = ["config ls -v", "proj ls -v", "env ls -v", "completions bash"]
        missing_proj = f"The '{proj_name}' project could not be found in your account."
        missing_env = f"The '{env_name}' environment could not be found in your account."

        # ensure not present
        result = self.run_cli(cmd_env, base_cmd + "proj ls")
        self.assertNotIn(proj_name, result.out())
        result = self.run_cli(cmd_env, base_cmd + "env ls")
        self.assertNotIn(env_name, result.out())

        ##############
        # Neither present
        eco_system = f"--project '{proj_name}' --env '{env_name}' "
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertNotEqual(result.return_value, 0)
            self.assertIn(missing_proj, result.err())
            self.assertIn(missing_env, result.err())

        for cmd in unchecked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertEqual(result.return_value, 0)
            self.assertNotIn(missing_proj, result.err())
            self.assertNotIn(missing_env, result.err())

        ##############
        # Project present, missing environment
        self.create_project(cmd_env, proj_name)
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertNotEqual(result.return_value, 0)
            self.assertNotIn(missing_proj, result.err())
            self.assertIn(missing_env, result.err())

        ##############
        # Environment present, missing project
        self.delete_project(cmd_env, proj_name)
        self.create_environment(cmd_env, env_name)
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertNotEqual(result.return_value, 0)
            self.assertIn(missing_proj, result.err())
            self.assertNotIn(missing_env, result.err())

        ##############
        # Both present
        self.create_project(cmd_env, proj_name)
        self.create_environment(cmd_env, env_name)
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertEqual(result.return_value, 0)
            self.assertNotIn(missing_proj, result.err())
            self.assertNotIn(missing_env, result.err())

        for cmd in unchecked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertEqual(result.return_value, 0)
            self.assertNotIn(missing_proj, result.err())
            self.assertNotIn(missing_env, result.err())

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

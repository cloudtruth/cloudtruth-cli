"""
Tests precedence of command line arguments, profiles(?), and environment variables.
"""
from testcase import TestCase, SRC_ARG, SRC_DEFAULT, SRC_PROFILE, SRC_ENV
from testcase import CT_ENV, CT_PROFILE, CT_PROJ, CT_TIMEOUT, CT_URL


class TestTopLevelArgs(TestCase):

    def test_arg_priority(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        printenv = f" run -i none -- {self.get_display_env_command()}"
        cfg_cmd = " config current -f csv"
        proj1 = self.make_name("test-arg-project-1")
        proj2 = self.make_name("test-arg-proj2")
        env1 = self.make_name("dev a")
        env2 = self.make_name("dev B")

        self.create_project(cmd_env, proj1)
        self.create_project(cmd_env, proj2)
        self.create_environment(cmd_env, env1)
        self.create_environment(cmd_env, env2)

        # remove things to make sure we have a "clean" environment
        cmd_env.pop(CT_PROJ, 'No project')
        cmd_env.pop(CT_ENV, 'No environment')

        # the CLOUDTRUTH_PROFILE cannot be removed, since it may change the server-url/api-key, but
        # need to accommodate for the profile also setting the project/environment variables
        def_proj = None
        def_env = "default"
        prof_name = cmd_env.get(CT_PROFILE, None)
        if prof_name:
            profile = self.get_profile(cmd_env, prof_name)
            if profile:
                def_proj = profile.get("Project", None)
                def_env = profile.get("Environment", None) or "default"

        # check defaults are used
        if def_proj:
            result = self.run_cli(cmd_env, base_cmd + printenv)
            self.assertResultSuccess(result)
            self.assertIn(f"{CT_PROJ}={def_proj}", result.out())
            self.assertIn(f"{CT_ENV}={def_env}", result.out())

            result = self.run_cli(cmd_env, base_cmd + cfg_cmd)
            self.assertResultSuccess(result)
            self.assertIn(f"Project,{def_proj},{SRC_PROFILE} ({prof_name})", result.out())
            self.assertIn(f"Environment,{def_env},{SRC_DEFAULT}", result.out())

        # set project/environment in environment
        cmd_env[CT_PROJ] = proj1
        cmd_env[CT_ENV] = env1

        # see items picked up from environment
        result = self.run_cli(cmd_env, base_cmd + printenv)
        self.assertResultSuccess(result)
        self.assertIn(f"{CT_PROJ}={proj1}", result.out())
        self.assertIn(f"{CT_ENV}={env1}", result.out())

        orig_timeout = cmd_env.pop(CT_TIMEOUT, None)
        orig_url = cmd_env.pop(CT_URL, None)
        timeout = "300"
        url = "https://127.0.0.2/bogus"
        cmd_env[CT_TIMEOUT] = timeout
        cmd_env[CT_URL] = url
        result = self.run_cli(cmd_env, base_cmd + cfg_cmd + " -x")
        self.assertResultSuccess(result)
        self.assertIn(f"Project,{proj1},{SRC_ENV}", result.out())
        self.assertIn(f"Environment,{env1},{SRC_ENV}", result.out())
        self.assertIn(f"Server URL,{url},{SRC_ENV}", result.out())
        self.assertIn(f"Request timeout,{timeout},{SRC_ENV}", result.out())
        cmd_env.pop(CT_URL)
        if orig_url:
            cmd_env[CT_URL] = orig_url
        cmd_env.pop(CT_TIMEOUT)
        if orig_timeout:
            cmd_env[CT_TIMEOUT] = orig_timeout

        # see that CLI arguments override the environment
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj2}' --env '{env2}'" + printenv)
        self.assertResultSuccess(result)
        self.assertIn(f"{CT_PROJ}={proj2}", result.out())
        self.assertIn(f"{CT_ENV}={env2}", result.out())

        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj2}' --env '{env2}'" + cfg_cmd + " -x")
        self.assertResultSuccess(result)
        self.assertIn(f"Project,{proj2},{SRC_ARG}", result.out())
        self.assertIn(f"Environment,{env2},{SRC_ARG}", result.out())

        # mix and match
        result = self.run_cli(cmd_env, base_cmd + f"--project '{proj2}'" + printenv)
        self.assertResultSuccess(result)
        self.assertIn(f"{CT_PROJ}={proj2}", result.out())
        self.assertIn(f"{CT_ENV}={env1}", result.out())

        result = self.run_cli(cmd_env, base_cmd + f"--env '{env2}'" + printenv)
        self.assertResultSuccess(result)
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
        proj_name = self.make_name("missing-subarg")
        self.create_project(cmd_env, proj_name)
        cmd_env[CT_PROJ] = proj_name

        for (subcmd, aliases) in {
            "actions": ["action", "act", "ac"],
            "actions imports": [
                "action import", "action imp", "act import", "act im", "act i", "ac imports",
            ],
            "actions pushes": [
                "action push", "action pu", "act pushes", "act pu", "act p",
            ],
            "audit-logs": ["audit", "aud", "au", "log", "logs"],
            "configuration": ["config", "conf", "con", "co", "c"],
            "configuration profiles": ["config profile", "conf prof", "c p"],
            "environments": ["environment", "envs", "env", "e"],
            "environments tag": ["environment tag", "env ta"],
            "import": ["imp", "im"],
            "integrations": ["integration", "integrate", "int"],
            "parameters": ["parameter", "params", "param", "par", "pa", "p"],
            "projects": ["project", "proj"],
            "run": ["ru", "r"],
            "templates": ["template", "te", "t"],
            "users": ["user", "us", "u"],
            "users invitations": ["user invitation", "us in", "u i"],
            "versions": ["version", "vers", "ver", "ve", "v"],
        }.items():
            for alias in [subcmd] + aliases:
                result = self.run_cli(cmd_env, base_cmd + alias)
                self.assertResultWarning(result, f"No '{subcmd}' sub-command executed")

        self.delete_project(cmd_env, proj_name)

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
        unchecked_commands = ["config prof ls -v", "proj ls -v", "env ls -v", "completions bash"]
        missing_proj = f"The '{proj_name}' project could not be found in your account."
        missing_env = f"The '{env_name}' environment could not be found in your account."

        # ensure not present -- csv is used to avoid errors when super-set names in use
        result = self.run_cli(cmd_env, base_cmd + "proj ls -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{proj_name},", result.out())
        result = self.run_cli(cmd_env, base_cmd + "env ls -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn(f"{env_name},", result.out())

        ##############
        # Neither present
        eco_system = f"--project '{proj_name}' --env '{env_name}' "
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertResultError(result, missing_proj)
            self.assertResultError(result, missing_env)

        for cmd in unchecked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertResultSuccess(result)

        ##############
        # Project present, missing environment
        self.create_project(cmd_env, proj_name)
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertResultError(result, missing_env)
            self.assertNotIn(missing_proj, result.err())

        ##############
        # Environment present, missing project
        self.delete_project(cmd_env, proj_name)
        self.create_environment(cmd_env, env_name)
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertResultError(result, missing_proj)
            self.assertNotIn(missing_env, result.err())

        ##############
        # Both present
        self.create_project(cmd_env, proj_name)
        for cmd in checked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertResultSuccess(result)

        for cmd in unchecked_commands:
            result = self.run_cli(cmd_env, base_cmd + eco_system + cmd)
            self.assertResultSuccess(result)

        # cleanup
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

    def test_arg_configurable_timeout(self):
        # NOTE: request_timeout is configurable via profile, but profiles are not integration tested
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        cmd = " project ls -v"

        cmd_env[CT_TIMEOUT] = "0"
        result = self.run_cli(cmd_env, base_cmd + cmd)
        self.assertResultError(result, "timed out")

    def test_arg_invalid_server(self):
        # NOTE: server_url is configurable via profile, but profiles are not integration tested
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        cmd = " projects ls -v"

        cmd_env[CT_URL] = "0.0.0.0:0"
        result = self.run_cli(cmd_env, base_cmd + cmd)
        self.assertResultError(result, "relative URL without a base")

        cmd_env[CT_URL] = "https://0.0.0.0:0/graphql"
        result = self.run_cli(cmd_env, base_cmd + cmd)
        self.assertResultError(result, "error trying to connect")

    def test_arg_authentication_errors(self):
        # NOTE: invalid key arguments override any profile or environment values.
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        commands = [
            "env ls -v",
            "param ls -v",
            "proj ls -v",
            "int ex -v",
            "int ls -v",
            f"run -i none -- {self.get_display_env_command()}",
        ]

        for user_cmd in commands:
            # test bogus key (means unauthenticated)
            result = self.run_cli(cmd_env, base_cmd + "--api-key abc123 " + user_cmd)
            self.assertResultError(result, "Not Authenticated")
            self.assertResultError(result, "Incorrect authentication credentials")

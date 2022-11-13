import os
from pathlib import Path

from testcase import CT_PROFILE, CT_API_KEY, PROP_MODIFIED
from testcase import TestCase


class TestRun(TestCase):

    def test_run_inheritance_env_only(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("run-env-proj")
        param_name = "SOME_PARAM_NAME"
        env_value = "env_value"

        sub_cmd = base_cmd + f"--project {proj_name} run "
        print_env = f" -c {self.get_display_env_command()}"

        self.create_project(cmd_env, proj_name)

        # add the value to the run environment, and see it does not get in without inheritance
        cmd_env[param_name] = env_value
        result = self.run_cli(cmd_env, sub_cmd + "--inherit none" + print_env)
        self.assertResultSuccess(result)
        self.assertNotIn(param_name, result.out())

        for inherit in ["underlay", "overlay", "exclusive"]:
            result = self.run_cli(cmd_env, sub_cmd + f"--inherit {inherit}" + print_env)
            self.assertResultSuccess(result)
            self.assertIn(f"{param_name}={env_value}", result.out())

        self.delete_project(cmd_env, proj_name)

    def test_run_inheritance_coordination(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("run-inherit-proj")
        param_name = "SOME_PARAM_NAME"
        env_value = self.make_name("env_value")
        ct_value = "ct_value"
        env_str = f"{param_name}={env_value}"
        ct_str = f"{param_name}={ct_value}"

        sub_cmd = base_cmd + f"--project {proj_name} run "
        print_env = f" -- {self.get_display_env_command()}"

        cmd_env[param_name] = env_value  # add to the run environment
        self.create_project(cmd_env, proj_name)
        self.set_param(cmd_env, proj_name, param_name, ct_value)

        result = self.run_cli(cmd_env, sub_cmd + "--inherit none" + print_env)
        self.assertResultSuccess(result)
        self.assertIn(ct_str, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--inherit underlay" + print_env)
        self.assertResultSuccess(result)
        self.assertIn(env_str, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--inherit overlay" + print_env)
        self.assertResultSuccess(result)
        self.assertIn(ct_str, result.out())

        # unspecified is the same as overlay
        result = self.run_cli(cmd_env, sub_cmd + print_env)
        self.assertResultSuccess(result)
        self.assertIn(ct_str, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--inherit exclusive" + print_env)
        self.assertResultError(result, f"Conflicting definitions in run environment for: {param_name}")

        self.delete_param(cmd_env, proj_name, param_name)
        self.delete_project(cmd_env, proj_name)

    def test_run_permissive(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("run-permissive-proj")
        sub_cmd = base_cmd + f"--project {proj_name} run "
        print_env = f" -- {self.get_display_env_command()}"

        # make sure we have something that normally gets removed
        if CT_API_KEY not in cmd_env:
            prof_name = cmd_env.get(CT_PROFILE, "default")
            profile = self.get_profile(cmd_env, prof_name)
            cmd_env[CT_API_KEY] = profile.get("API")

        self.create_project(cmd_env, proj_name)

        result = self.run_cli(cmd_env, sub_cmd + print_env)
        self.assertResultSuccess(result)
        self.assertNotIn(CT_API_KEY, result.out())

        result = self.run_cli(cmd_env, sub_cmd + "--permissive" + print_env)
        self.assertResultSuccess(result)
        self.assertIn(CT_API_KEY, result.out())

        self.delete_project(cmd_env, proj_name)

    def test_run_arg_with_spaces(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("run-spaces")
        printenv = self.get_display_env_command()

        self.create_project(cmd_env, proj_name)

        # This hits the case where the command may (or may not run)
        filename = "temp.txt"
        cmd = base_cmd + f"--project {proj_name} run  -i none -- '{printenv} > {filename}' {printenv}"

        result = self.run_cli(cmd_env, cmd)
        # NOTE: whether this passes or not may depend on platform, so cannot use assertResultXxx()
        self.assertIn("command contains spaces, and may fail", result.err())

        # cleanup
        file = Path(os.getcwd()) / f"{filename}"
        file.unlink(missing_ok=True)  # may not have created the file, but just in case we did
        self.delete_project(cmd_env, proj_name)

    def test_run_time(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("proj-old-run")
        param_name = "my_param"
        first_value = "first-value"
        second_value = "second-value"
        printenv = self.get_display_env_command()
        run_cmd = base_cmd + f"--project '{proj_name}' run "

        self.create_project(cmd_env, proj_name)
        self.set_param(cmd_env, proj_name, param_name, first_value)

        # get modified time
        details = self.get_param(cmd_env, proj_name, param_name)
        orig_modified = details.get(PROP_MODIFIED)

        self.set_param(cmd_env, proj_name, param_name, second_value)

        # run with the time specified
        result = self.run_cli(cmd_env, run_cmd + f"--as-of {orig_modified} -- {printenv}")
        self.assertResultSuccess(result)
        self.assertIn(first_value, result.out())
        self.assertNotIn(second_value, result.out())

        # run again without the time specified, and see the first value
        result = self.run_cli(cmd_env, run_cmd + f"-- {printenv}")
        self.assertResultSuccess(result)
        self.assertNotIn(first_value, result.out())
        self.assertIn(second_value, result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)

    def test_run_strict(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("run-strict")
        printenv = self.get_display_env_command()
        param_name = "SOME_PARAM_NAME"
        param_value = "some-value"

        param_cmd = base_cmd + f"--project {proj_name} parameters set {param_name}"
        self.create_project(cmd_env, proj_name)
        self.run_cli(cmd_env, param_cmd)

        # assert failure when a cloudtruth parameter has no value
        cmd = base_cmd + f"--project {proj_name} run --strict -- {printenv}"
        result = self.run_cli(cmd_env, cmd)
        self.assertIn("parameter found without a value", result.err())

        # assert success when all cloudtruth parameters have values
        value_cmd = param_cmd + f" --value {param_value}"
        self.run_cli(cmd_env, value_cmd)
        result = self.run_cli(cmd_env, cmd)
        self.assertIn(param_name, result.out())

        # cleanup
        self.delete_project(cmd_env, proj_name)

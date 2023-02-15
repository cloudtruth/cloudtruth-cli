import subprocess

from testcase import TestCase
from testcase import CT_API_KEY
from testcase import CT_PROJ
from testcase import CT_ENV
from testcase import CT_PROFILE
from testcase import CT_REST_DEBUG
from testcase import CT_REST_PAGE_SIZE
from testcase import CT_REST_SUCCESS
from testcase import CT_TIMEOUT
from testcase import REDACTED
from testcase import find_by_prop


class TestConfiguration(TestCase):
    basic_prof_name = "cli-int-basic-prof-test"
    basic_child_prof = "cli-int-basic-child-test"
    current_prof_name = "cli-int-curr-prof-test"

    def tearDown(self) -> None:
        # clean up any stranded profiles
        for profile in [self.basic_prof_name, self.current_prof_name, self.basic_child_prof]:
            cmd = self.get_cli_base_cmd() + f"profile delete -y '{profile}'"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        super().tearDown()

    def test_configuration_profile(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        conf_cmd = base_cmd + "conf "
        list_cmd = conf_cmd + "prof ls "
        set_cmd = conf_cmd + "profile set "
        prof_name = self.basic_prof_name
        child_name = self.basic_child_prof
        desc1 = "Profile to use for something"
        desc2 = "alternate description"
        api_key1 = "bogus-key-value"
        env1 = "some-environment"
        proj1 = "my-proj-name"

        # make sure it is not already present
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertNotIn(prof_name, result.out())

        # verify a good message for a missing profile
        result = self.run_cli(cmd_env, base_cmd + f"--profile '{prof_name}' env ls -v")
        self.assertResultError(result, f"Failed to load configuration from profile '{prof_name}'")

        # create it
        result = self.run_cli(cmd_env, set_cmd + f"'{prof_name}' -k '{api_key1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Created profile '{prof_name}'", result.out())

        # verify it was created
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertIn(prof_name, result.out())

        # update some fields
        result = self.run_cli(cmd_env, set_cmd + f"'{prof_name}' -d '{desc1}' -e '{env1}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated profile '{prof_name}'", result.out())

        # make sure API key is not shown, but other parameters are
        result = self.run_cli(cmd_env, list_cmd + "-vf csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{prof_name},{REDACTED},{env1},,{desc1}", result.out())

        # update some fields
        result = self.run_cli(cmd_env, set_cmd + f"'{prof_name}' -p '{proj1}' -d '{desc2}'")
        self.assertResultSuccess(result)

        # make sure API key is not shown, but other parameters are
        result = self.run_cli(cmd_env, list_cmd + "-svf csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{prof_name},{api_key1},{env1},{proj1},{desc2}", result.out())

        # update without any properties produces warning
        result = self.run_cli(cmd_env, set_cmd + f"'{prof_name}'")
        self.assertResultWarning(result, f"Nothing to change for profile '{prof_name}'")

        profiles = self.get_cli_entries(cmd_env, list_cmd + "-svf json", "profile")
        prof = [p for p in profiles if p.get("Name") == prof_name][0]
        self.assertEqual(api_key1, prof.get("API"))
        self.assertEqual(env1, prof.get("Environment"))
        self.assertEqual(proj1, prof.get("Project"))
        self.assertEqual(desc2, prof.get("Description"))

        # create a child
        child_project = "this-is-a-bogus-project-name"
        result = self.run_cli(cmd_env, set_cmd + f"'{child_name}' -s '{prof_name}' -p '{child_project}'")
        self.assertResultSuccess(result)

        # verify child was created
        result = self.run_cli(cmd_env, list_cmd + "-v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(child_name, result.out())
        self.assertIn(child_project, result.out())

        # see that creating with a non-existent source/parent fails
        missing_child = "cli-int-prof-missing-child"
        new_child = "cli-int-prof-never-created"
        result = self.run_cli(cmd_env, set_cmd + f"'{new_child}' -s '{missing_child}'")
        self.assertResultError(result, f"Source profile '{missing_child}' does not exist")

        # delete the child
        result = self.run_cli(cmd_env, conf_cmd + f"p d -y '{child_name}'")
        self.assertResultSuccess(result)

        # delete it
        result = self.run_cli(cmd_env, conf_cmd + f"p d -y '{prof_name}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Deleted profile '{prof_name}'", result.out())

        # verify it is gone
        result = self.run_cli(cmd_env, list_cmd)
        self.assertResultSuccess(result)
        self.assertNotIn(prof_name, result.out())

        # see deletion is idempotent
        result = self.run_cli(cmd_env, conf_cmd + f"profile delete -y '{prof_name}'")
        self.assertResultWarning(result, f"Profile '{prof_name}' does not exist")

    def test_configuration_current(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        conf_cmd = base_cmd + "conf "
        curr_cmd = conf_cmd + "curr "

        prof_name = self.current_prof_name
        api_key = "bogus-key-value"
        env_name = "some-environment"
        proj_name = "my-proj-name"
        set_cmd = conf_cmd + f"prof set '{prof_name}' "

        result = self.run_cli(cmd_env, curr_cmd)
        self.assertResultSuccess(result)
        orig_env = result.out()

        # create the profile
        result = self.run_cli(cmd_env, set_cmd + f"-k '{api_key}' -e '{env_name}' -p '{proj_name}'")
        self.assertResultSuccess(result)

        # nothing changed, since did not setup to use new profile
        result = self.run_cli(cmd_env, curr_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(orig_env, result.out())

        # now, set the environment to use the profile, and remove other environmental stuff
        cmd_env[CT_PROFILE] = prof_name
        cmd_env.pop(CT_API_KEY, None)
        cmd_env.pop(CT_PROJ, None)
        cmd_env.pop(CT_ENV, None)

        # see that things have changed
        result = self.run_cli(cmd_env, curr_cmd)
        self.assertResultSuccess(result)
        self.assertNotEqual(orig_env, result.out())
        self.assertIn(prof_name, result.out())

        # check the "all" parameters
        profile = self.get_cli_entries(cmd_env, curr_cmd + "-sf json", "profile")
        this_profile = f"profile ({prof_name})"
        param_names = [e.get("Parameter") for e in profile]
        expected = ["Profile", "API key", "Organization", "User", "Role", "Project", "Environment"]
        self.assertEqual(param_names, expected)

        entry = find_by_prop(profile, "Parameter", "Profile")[0]
        self.assertEqual(entry.get("Value"), prof_name)
        self.assertEqual(entry.get("Source"), "shell")

        entry = find_by_prop(profile, "Parameter", "API key")[0]
        self.assertEqual(entry.get("Value"), api_key)
        self.assertEqual(entry.get("Source"), this_profile)

        entry = find_by_prop(profile, "Parameter", "Project")[0]
        self.assertEqual(entry.get("Value"), proj_name)
        self.assertEqual(entry.get("Source"), this_profile)

        entry = find_by_prop(profile, "Parameter", "Environment")[0]
        self.assertEqual(entry.get("Value"), env_name)
        self.assertEqual(entry.get("Source"), this_profile)

        entry = find_by_prop(profile, "Parameter", "Organization")[0]
        self.assertEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "")

        entry = find_by_prop(profile, "Parameter", "User")[0]
        self.assertEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "")

        entry = find_by_prop(profile, "Parameter", "Role")[0]
        self.assertEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "")

        ##############################
        # extended version
        cmd_env[CT_REST_SUCCESS] = "a,b,c,d"
        cmd_env[CT_REST_DEBUG] = "false"
        cmd_env[CT_REST_PAGE_SIZE] = "9"
        cmd_env[CT_TIMEOUT] = "33"
        profile = self.get_cli_entries(cmd_env, curr_cmd + "-xsf json", "profile")
        this_profile = f"profile ({prof_name})"
        param_names = [e.get("Parameter") for e in profile]
        extended = [
            "Profile",
            "API key",
            "Organization",
            "User",
            "Role",
            "Project",
            "Environment",
            "CLI version",
            "Server URL",
            "Request timeout",
            "REST debug",
            "REST success",
            "REST page size",
            "Accept Invalid Certs",
        ]
        self.assertEqual(param_names, extended)

        entry = find_by_prop(profile, "Parameter", "REST debug")[0]
        self.assertEqual(entry.get("Value"), "false")
        self.assertEqual(entry.get("Source"), "shell")

        entry = find_by_prop(profile, "Parameter", "REST success")[0]
        self.assertEqual(entry.get("Value"), "a, b, c, d")
        self.assertEqual(entry.get("Source"), "shell")

        entry = find_by_prop(profile, "Parameter", "REST page size")[0]
        self.assertEqual(entry.get("Value"), "9")
        self.assertEqual(entry.get("Source"), "shell")

        entry = find_by_prop(profile, "Parameter", "Request timeout")[0]
        self.assertEqual(entry.get("Value"), "33")
        self.assertEqual(entry.get("Source"), "shell")

        cmd_env.pop(CT_REST_SUCCESS)
        cmd_env.pop(CT_REST_DEBUG)
        cmd_env.pop(CT_REST_PAGE_SIZE)
        cmd_env.pop(CT_TIMEOUT)

        ##############################
        # test with command line arguments
        bogus_api_key = "not-a-real-api-key"
        cmd = base_cmd + f"--api-key '{bogus_api_key}' --profile '{prof_name}' conf curr -sf json"
        profile = self.get_cli_entries(cmd_env, cmd, "profile")

        entry = find_by_prop(profile, "Parameter", "API key")[0]
        self.assertEqual(entry.get("Value"), bogus_api_key)
        self.assertEqual(entry.get("Source"), "argument")

        entry = find_by_prop(profile, "Parameter", "Profile")[0]
        self.assertEqual(entry.get("Value"), prof_name)
        self.assertEqual(entry.get("Source"), "argument")

        entry = find_by_prop(profile, "Parameter", "Organization")[0]
        self.assertEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "")

        entry = find_by_prop(profile, "Parameter", "User")[0]
        self.assertEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "")

        entry = find_by_prop(profile, "Parameter", "Role")[0]
        self.assertEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "")

        ##############################
        # delete the profile
        result = self.run_cli(cmd_env, conf_cmd + f"p d -y '{prof_name}'")
        self.assertResultSuccess(result)

        # when profile is not found, command succeeds without the bits from the config
        result = self.run_cli(cmd_env, curr_cmd + "-s")
        self.assertResultSuccess(result)
        self.assertIn(prof_name, result.out())
        self.assertNotIn(api_key, result.out())
        self.assertNotIn(env_name, result.out())
        self.assertNotIn(proj_name, result.out())
        self.assertNotIn(this_profile, result.out())

        ##############################
        # back to the original user values  -- do not show secrets
        cmd_env = self.get_cmd_env()
        profile = self.get_cli_entries(cmd_env, base_cmd + "conf curr -f json", "profile")
        entry = find_by_prop(profile, "Parameter", "API key")[0]
        self.assertEqual(entry.get("Value"), REDACTED)
        entry = find_by_prop(profile, "Parameter", "Organization")[0]
        self.assertNotEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "API key")
        entry = find_by_prop(profile, "Parameter", "User")[0]
        self.assertNotEqual(entry.get("Value"), "")
        self.assertEqual(entry.get("Source"), "API key")
        entry = find_by_prop(profile, "Parameter", "Role")[0]
        self.assertIn(entry.get("Value"), ["owner", "admin", "contrib"])
        self.assertEqual(entry.get("Source"), "API key")

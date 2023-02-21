import dataclasses
import os
import shlex
import subprocess
import unittest
import re
from copy import deepcopy
from datetime import datetime
from datetime import timedelta
from pathlib import Path
from typing import Dict
from typing import List
from typing import Optional


# These are environment variable names used by the application
CT_API_KEY = "CLOUDTRUTH_API_KEY"
CT_ENV = "CLOUDTRUTH_ENVIRONMENT"
CT_PROFILE = "CLOUDTRUTH_PROFILE"
CT_PROJ = "CLOUDTRUTH_PROJECT"
CT_URL = "CLOUDTRUTH_SERVER_URL"
CT_TIMEOUT = "CLOUDTRUTH_REQUEST_TIMEOUT"
CT_REST_DEBUG = "CLOUDTRUTH_REST_DEBUG"
CT_REST_SUCCESS = "CLOUDTRUTH_REST_SUCCESS"
CT_REST_PAGE_SIZE = "CLOUDTRUTH_REST_PAGE_SIZE"

DEFAULT_SERVER_URL = "https://api.cloudtruth.io"
DEFAULT_ENV_NAME = "default"
DEFAULT_PROFILE_NAME = "default"

AUTO_DESCRIPTION = "Automated testing via live_test"
TEST_PAGE_SIZE = 5

CT_TEST_LOG_COMMANDS = "CT_LIVE_TEST_LOG_COMMANDS"
CT_TEST_LOG_OUTPUT = "CT_LIVE_TEST_LOG_OUTPUT"
CT_TEST_LOG_COMMANDS_ON_FAILURE = "CT_LIVE_TEST_LOG_COMMANDS_ON_FAILURE"
CT_TEST_LOG_OUTPUT_ON_FAILURE = "CT_LIVE_TEST_LOG_OUTPUT_ON_FAILURE"
CT_TEST_JOB_ID = "CT_LIVE_TEST_JOB_ID"
CT_TEST_KNOWN_ISSUES = "CT_LIVE_TEST_KNOWN_ISSUES"

SRC_ENV = "shell"
SRC_ARG = "argument"
SRC_PROFILE = "profile"
SRC_DEFAULT = "default"

REDACTED = "*****"
DEFAULT_PARAM_VALUE = "-"

# properties
PROP_CREATED = "Created At"
PROP_DESC = "Description"
PROP_MODIFIED = "Modified At"
PROP_NAME = "Name"
PROP_RAW = "Raw"
PROP_TYPE = "Type"
PROP_VALUE = "Value"

REGEX_REST_DEBUG = re.compile("^URL \\w+ .+? elapsed: [\\d\\.]+\\w+$")


def get_cli_base_cmd() -> str:
    """
    This is a separate function that does not reference the `self._base_cmd' so it can be called
    during __init__(). It returns the path to the executable (presumably) with the trailing
    space to allow for easier consumption.
    """
    # walk back up looking for top of projects, and goto `target/debug/cloudtruth`
    curr = Path(__file__).absolute()
    exec_name = "cloudtruth.exe" if os.name == "nt" else "cloudtruth"
    exec_path_release = Path("target") / "release" / exec_name
    exec_path_debug = Path("target") / "debug" / exec_name

    # leverage current structure... walk back up a maximum of 2 levels
    for _ in range(3):
        possible_debug = curr.parent / exec_path_debug
        possible_release = curr.parent / exec_path_release
        # print(possible_debug, possible_release, sep="\n")
        # prefer latest build if both exist
        if possible_debug.exists() and possible_release.exists():
            if os.path.getmtime(possible_debug) > os.path.getmtime(possible_release):
                return str(possible_debug) + " "
            else:
                return str(possible_release) + " "
        if possible_debug.exists():
            return str(possible_debug) + " "
        if possible_release.exists():
            return str(possible_release) + " "
        curr = curr.parent

    # we failed to find this, so just use the "default".
    return exec_name + " "


def find_by_prop(entries: List[Dict], prop_name: str, prop_value: str) -> List[Dict]:
    return [e for e in entries if e.get(prop_name, None) == prop_value]


def missing_any(env_var_names: List[str]) -> bool:
    return not all([os.environ.get(x) for x in env_var_names])


# decorator to mark a test as a known issue
def skip_known_issue(msg: str):
    return unittest.skipUnless(os.environ.get(CT_TEST_KNOWN_ISSUES), f"Known issue: {msg}")


@dataclasses.dataclass
class Result:
    return_value: int = 0
    stdout: List = dataclasses.field(default_factory=list)
    stderr: List = dataclasses.field(default_factory=list)
    timediff: timedelta = timedelta(0)
    command: Optional[str] = None

    def out(self) -> str:
        return "\n".join(self.stdout)

    def err(self) -> str:
        return "\n".join(self.stderr)

    def out_contains(self, needle: str) -> Optional[str]:
        for line in self.stdout:
            if needle in line:
                return line
        return None

    def all(self) -> str:
        return self.out() + "\n" + self.err()


class TestCase(unittest.TestCase):
    """
    This extends the unittest.TestCase to add some basic functions
    """

    def __init__(self, *args, **kwargs):
        self._base_cmd = get_cli_base_cmd()
        self.log_commands = int(os.environ.get(CT_TEST_LOG_COMMANDS, "0"))
        self.log_output = int(os.environ.get(CT_TEST_LOG_OUTPUT, "0"))
        self.log_commands_on_failure = int(os.environ.get(CT_TEST_LOG_COMMANDS_ON_FAILURE, "0"))
        self.log_output_on_failure = int(os.environ.get(CT_TEST_LOG_OUTPUT_ON_FAILURE, "0"))
        self.job_id = os.environ.get(CT_TEST_JOB_ID)
        self.rest_debug = os.environ.get(CT_REST_DEBUG, "False").lower() in ("true", "1", "y", "yes")
        self._failure_logs = None
        self._projects = None
        self._environments = None
        self._users = None
        self._invites = None
        self._types = None
        self._filenames = None
        self._groups = None
        super().__init__(*args, **kwargs)
        self.maxDiff = None

    def setUp(self) -> None:
        # collects logs to display when/if the test case fails
        self._failure_logs = list()
        # start each test with empty sets for projects and environments
        self._projects = list()
        self._environments = list()
        self._users = list()
        self._invites = list()
        self._types = list()
        self._filenames = set()
        self._groups = list()
        super().setUp()

    def tearDown(self) -> None:
        # Report test failures
        if not self.log_commands and self.log_commands_on_failure or not self.log_output and self.log_output_on_failure:
            # Python 3.4 - 3.10
            if hasattr(self._outcome, "errors"):
                result = self.defaultTestResult()
                self._feedErrorsToResult(result, self._outcome.errors)
            # Python 3.11+
            else:
                result = self._outcome.result
            success = all(test != self for test, _ in result.errors + result.failures)
            if not success:
                print()  # gives better reading output
                print("\n".join(self._failure_logs))

        # tear down any possibly lingering projects -- they should have been deleted in reverse
        # order in case there are any children.
        for proj in reversed(self._projects):
            cmd = self._base_cmd + f'proj del "{proj}" --confirm'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # tear down any possibly lingering environments -- they should have been deleted in reverse
        # order in case there are any children.
        for env in reversed(self._environments):
            cmd = self._base_cmd + f'env del "{env}" --confirm'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # delete any possibly lingering users
        for usr in self._users:
            cmd = self._base_cmd + f'user del --confirm "{usr}"'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # delete any possibly lingering invitations
        for email in self._invites:
            cmd = self._base_cmd + f'user invitations del --confirm "{email}"'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # tear down any possibly lingering types -- they should have been deleted in reverse
        # order in case there are any children.
        for typename in reversed(self._types):
            cmd = self._base_cmd + f'type del "{typename}" --confirm'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # remove any added files
        for fname in self._filenames:
            os.remove(fname)

        # remove any added groups
        for groupname in self._groups:
            cmd = self._base_cmd + f'group del "{groupname}" --confirm'
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        super().tearDown()

    def write_file(self, filename: str, content: str) -> None:
        """
        Utility to open set the filename content, and save the name in the list
        for deletion.
        """
        self._filenames.add(filename)
        file = open(filename, "w")
        file.write(content)
        file.close()

    def delete_file(self, filename):
        self._filenames.remove(filename)
        os.remove(filename)

    def make_name(self, name: str) -> str:
        """
        Adds the JOB_ID to the name if present, so multiple tests can run simultaneously.
        """
        if not self.job_id:
            return name
        return name + "-" + self.job_id

    def get_cli_base_cmd(self) -> str:
        """
        Finds where to get the executable image from.
        The result includes an extra space, and whatever other args may be necessary (e.g. api_key)
        """
        if not self._base_cmd:
            self._base_cmd = get_cli_base_cmd()
        return self._base_cmd

    def get_cmd_env(self):
        env_copy = deepcopy(os.environ)
        ## temporarily unset the CLOUDTRUTH_REST_DEBUG environment variable if defined, so that
        ## in run_cli_cmd() we can detect if a test explicitly set it. this allows us to determine if
        ## we should keep the debug logs in stdout for tests that explicitly assert on them (ex: test_timing.py),
        ## or if we should strip debug logs from stdout to prevent assertion failures in tests that are not
        ## expecting debug logs.
        if env_copy.get(CT_REST_DEBUG, "false").lower() in ("true", "1", "y", "yes"):
            del env_copy[CT_REST_DEBUG]
        return env_copy

    def get_display_env_command(self) -> str:
        if os.name == "nt":
            return "SET"
        return "printenv"

    def assertResultSuccess(self, result: Result, success_msg: Optional[str] = None):
        """
        This is a convenience method to check the return code, and error output.
        """
        # check the error message is empty first, since it gives the most info about a failure
        self.assertEqual(result.err(), "")
        self.assertEqual(result.return_value, 0)
        if success_msg:
            self.assertIn(success_msg, result.out())

    def assertResultWarning(self, result: Result, warn_msg: str):
        """
        This is a convenience method to check for successful CLI commands that emit a (partial) warning message
        """
        # check the message first, since it is more telling when the command fails
        self.assertIn(warn_msg, result.err())
        self.assertEqual(result.return_value, 0)

    def assertResultError(self, result: Result, err_msg: str):
        """
        This is a convenience method to check for failed CLI commands with a specific (partial) error message
        """
        self.assertIn(err_msg, result.err())
        self.assertNotEqual(result.return_value, 0)

    def assertResultIn(self, result: Result, needle: str):
        """
        This is a convenience method to check for the needle in either stdout or stderr
        """
        self.assertIn(needle, result.all())

    def assertPaginated(self, cmd_env, command: str, in_req: str, page_size: int = TEST_PAGE_SIZE):
        """
        Sets an artificially low CLOUDTRUTH_REST_PAGE_SIZE so we get paginated results for the
        provided command, and checks the output includes the URLs that specify additional pages.
        """
        local_env = deepcopy(cmd_env)
        local_env[CT_REST_DEBUG] = "true"
        local_env[CT_REST_PAGE_SIZE] = str(page_size)
        result = self.run_cli(local_env, command)
        self.assertResultSuccess(result)
        gets = [_ for _ in result.stdout if "URL GET" in _ and in_req in _]
        size_search = f"page_size={page_size}"
        size_spec = [_ for _ in gets if size_search in _]
        self.assertGreaterEqual(len(size_spec), 2)  # should have at least 2 paginated requests
        self.assertGreaterEqual(len([_ for _ in gets if "page=1" in _]), 1)
        self.assertGreaterEqual(len([_ for _ in gets if "page=2" in _]), 1)

    def run_cli(self, env: Dict[str, str], cmd: str) -> Result:  # noqa: C901
        # WARNING: DOS prompt does not like the single quotes, so use double
        cmd = cmd.replace("'", '"')

        if self.log_commands:
            print(cmd)
        elif self.log_commands_on_failure:
            self._failure_logs.append(cmd)

        def _next_part(arg_list: List, key: str) -> str:
            """Simple function to walk the 'arg_list' and find the item after the 'key'"""
            for index, value in enumerate(arg_list):
                if value == key:
                    return arg_list[index + 1]
            return None

        # split the command args into something we can work with
        args = shlex.split(cmd)
        if "set" in args:
            # if we're using any of our 'environments' aliases
            if set(args) & set(["environments", "environment", "envs", "env", "e"]):
                env_name = _next_part(args, "set")
                if env_name and env_name not in self._environments:
                    self._environments.append(env_name)
                env_name = _next_part(args, "--rename") or _next_part(args, "-r")
                if env_name and env_name not in self._environments:
                    self._environments.append(env_name)
            # if we're using any of our 'projects' aliases
            elif set(args) & set(["projects", "project", "proj"]):
                proj_name = _next_part(args, "set")
                if proj_name and proj_name not in self._projects:
                    self._projects.append(proj_name)
                proj_name = _next_part(args, "--rename") or _next_part(args, "-r")
                if proj_name and proj_name not in self._projects:
                    self._projects.append(proj_name)
            elif set(args) & set(["users", "user", "us"]):
                user_name = _next_part(args, "set")
                if user_name and user_name not in self._users:
                    self._users.append(user_name)
            elif set(args) & set(["invitations", "invites", "invite", "inv", "in"]):
                email = _next_part(args, "set")
                if email and email not in self._invites:
                    self._invites.append(email)
            elif set(args) & set(["parameter-types", "param-types", "param-type", "types", "type", "ty"]):
                typename = _next_part(args, "set")
                if typename and typename not in self._types:
                    self._types.append(typename)
            elif set(args) & set(["group", "grp", "gr", "g"]):
                groupname = _next_part(args, "set")
                if groupname and groupname not in self._groups:
                    self._groups.append(groupname)

        ## determine if we should strip REST debug logs from the command output. note that in get_cmd_env() we remove
        ## CLOUDTRUTH_REST_DEBUG variable from the local copy. this makes it possible to detect if a test case
        ## explicitly set it and thus wants the debug logs in its output
        orig_rest_debug_value = env.get(CT_REST_DEBUG)
        strip_rest_debug = self.rest_debug and not orig_rest_debug_value
        if strip_rest_debug:
            env[CT_REST_DEBUG] = "true"

        start = datetime.now()
        process = subprocess.run(cmd, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        delta = datetime.now() - start
        result = Result(
            return_value=process.returncode,
            stdout=process.stdout.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
            stderr=process.stderr.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
            timediff=delta,
            command=cmd,
        )

        ## Log outputs
        ## TODO: may want to consider using TextTestRunners buffer option for log-on-failure behavior)
        if self.log_output:
            if result.stdout:
                print("\n".join(result.stdout))
            if result.stderr:
                print("\n".join(result.stderr))
        elif self.log_output_on_failure:
            if result.stdout:
                self._failure_logs.append("\n".join(result.stdout))
            if result.stderr:
                self._failure_logs.append("\n".join(result.stderr))
        elif self.rest_debug:
            debug_out = [line for line in result.stdout if re.match(REGEX_REST_DEBUG, line)]
            if debug_out:
                print("\n".join(debug_out))

        if strip_rest_debug:
            ## if stripping debug output, re-enable original CLOUDTRUTH_REST_DEBUG value if previously found
            if orig_rest_debug_value is not None:
                env[CT_REST_DEBUG] = orig_rest_debug_value
            else:
                del env[CT_REST_DEBUG]
            ## now strip logs from output before returning. do this after the logging steps above so that console has
            ## complete logs, but test cases have stripped logs
            result.stdout = [line for line in result.stdout if not re.match(REGEX_REST_DEBUG, line)]

        return result

    def add_environment_for_cleanup(self, env_name: str) -> None:
        if env_name not in self._environments:
            self._environments.append(env_name)

    def add_project_for_cleanup(self, proj_name: str):
        if proj_name not in self._projects:
            self._projects.append(proj_name)

    def get_cli_entries(self, env: Dict[str, str], cmd: str, label: str) -> Optional[List[Dict]]:
        result = self.run_cli(env, cmd)
        self.assertResultSuccess(result)
        if result.out().startswith("No "):
            return []
        return eval(result.out()).get(label)

    def get_profile(self, cmd_env, prof_name: str) -> Optional[Dict]:
        result = self.run_cli(cmd_env, self._base_cmd + "config prof list --values --format csv -s")
        self.assertResultSuccess(result)
        needle = f"{prof_name},"
        for line in result.stdout:
            if line.startswith(needle):
                values = line.split(",")
                return {
                    "Name": values[0],
                    "API": values[1],
                    "Environment": values[2],
                    "Project": values[3],
                    "Description": values[4],
                }
        return None

    def get_current_config(self, cmd_env, property_name: str) -> Optional[str]:
        result = self.run_cli(cmd_env, self._base_cmd + "config curr --format json")
        self.assertResultSuccess(result)
        profile_props = eval(result.out()).get("profile", [])
        for prop in profile_props:
            if prop.get("Parameter") == property_name:
                return prop.get("Value", None)
        return None

    def create_project(self, cmd_env, proj_name: str, parent: Optional[str] = None) -> Result:
        proj_cmd = self._base_cmd + f"proj set '{proj_name}' -d '{AUTO_DESCRIPTION}' "
        if parent:
            proj_cmd += f"--parent '{parent}'"
        result = self.run_cli(cmd_env, proj_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Created project '{proj_name}'", result.out())
        return result

    def delete_project(self, cmd_env, proj_name: str) -> Result:
        result = self.run_cli(cmd_env, self._base_cmd + f"proj delete '{proj_name}' --confirm")
        self.assertResultSuccess(result)
        return result

    def create_environment(self, cmd_env, env_name: str, parent: Optional[str] = None) -> Result:
        cmd = self._base_cmd + f"env set '{env_name}' "
        if parent:
            cmd += f"-p '{parent}' "
        cmd += f"-d '{AUTO_DESCRIPTION}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Created environment '{env_name}'", result.out())
        return result

    def delete_environment(self, cmd_env, env_name: str) -> Result:
        result = self.run_cli(cmd_env, self._base_cmd + f"env del '{env_name}' --confirm")
        self.assertResultSuccess(result)
        return result

    def create_type(
        self,
        cmd_env,
        type_name: str,
        parent: Optional[str] = None,
        extra: Optional[str] = None,
    ) -> Result:
        type_cmd = self._base_cmd + f"param-type set '{type_name}' -d '{AUTO_DESCRIPTION}' "
        if parent:
            type_cmd += f"--parent '{parent}' "
        if extra:
            type_cmd += extra
        result = self.run_cli(cmd_env, type_cmd)
        self.assertResultSuccess(result)
        self.assertIn(f"Created parameter type '{type_name}'", result.out())
        return result

    def delete_type(self, cmd_env, type_name: str) -> Result:
        result = self.run_cli(cmd_env, self._base_cmd + f"param-type delete '{type_name}' --confirm")
        self.assertResultSuccess(result)
        return result

    def create_env_tag(
        self, cmd_env, env_name: str, tag_name: str, desc: Optional[str] = None, time: Optional[str] = None
    ) -> None:
        cmd = self._base_cmd + f"env tag set '{env_name}' '{tag_name}' "
        if desc:
            cmd += f"--desc '{desc}'"
        if time:
            cmd += f"--time '{time}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Created", result.out())

    def delete_env_tag(self, cmd_env, env_name: str, tag_name: str) -> None:
        cmd = self._base_cmd + f"env tag del '{env_name}' '{tag_name}' -y "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn("Deleted", result.out())

    def set_param(
        self,
        cmd_env,
        proj: str,
        name: str,
        value: Optional[str] = None,
        secret: Optional[bool] = None,
        env: Optional[str] = None,
        desc: Optional[str] = None,
        param_type: Optional[str] = None,
        fqn: Optional[str] = None,
        jmes: Optional[str] = None,
        evaluate: Optional[bool] = None,
        extra: Optional[str] = None,
    ) -> Result:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += f"param set '{name}' "
        if value:
            cmd += f"--value '{value}' "
        if secret is not None:
            cmd += f"--secret '{str(secret).lower()}' "
        if desc:
            cmd += f"--desc '{desc}' "
        if param_type:
            cmd += f"--type '{param_type}' "
        if fqn:
            cmd += f"--fqn '{fqn}' "
        if jmes:
            cmd += f"--jmes '{jmes}' "
        if evaluate is not None:
            cmd += f"--evaluate '{str(evaluate).lower()}' "
        if extra:
            cmd += extra
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        return result

    def get_param(
        self,
        cmd_env,
        proj: str,
        name: str,
        env: Optional[str] = None,
        secrets: Optional[bool] = None,
        as_of: Optional[str] = None,
    ) -> Optional[Dict]:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += "param list --show-times --format json "
        if as_of:
            cmd += f"--as-of '{as_of}' "
        if secrets:
            cmd += "--secrets "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        parameters = eval(result.out())
        for item in parameters["parameter"]:
            if item.get("Name") == name:
                return item
        return None

    def unset_param(
        self,
        cmd_env,
        proj: str,
        name: str,
        env: Optional[str] = None,
    ) -> Result:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += f"param unset '{name}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        return result

    def delete_param(self, cmd_env, proj: str, name: str, env: Optional[str] = None) -> Result:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += f"param delete -y '{name}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        return result

    def verify_param(
        self,
        cmd_env,
        proj: str,
        name: str,
        value: str,
        env: Optional[str] = None,
        as_of: Optional[str] = None,
    ):
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        # check the output using the 'get' command
        cmd += f"param get '{name}' "
        if as_of:
            cmd += f"--as-of '{as_of}' "

        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        self.assertIn(value, result.out())

    def list_params(
        self,
        cmd_env,
        proj: str,
        env: Optional[str] = None,
        show_values: bool = True,
        secrets: bool = False,
        fmt: Optional[str] = None,
        as_of: Optional[str] = None,
        show_times: bool = False,
        show_rules: bool = False,
        show_external: bool = False,
        show_evaluated: bool = False,
        show_parents: bool = False,
        show_children: bool = False,
    ) -> Result:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += "param ls "
        if fmt:
            cmd += f"-f {fmt} "
        if as_of:
            cmd += f"--as-of '{as_of}' "
        if secrets:
            cmd += "-s "
        if show_values:
            cmd += "-v "
        if show_times:
            cmd += "--show-times "
        if show_rules:
            cmd += "--rules "
        if show_external:
            cmd += "--external "
        if show_evaluated:
            cmd += "--evaluated "
        if show_parents:
            cmd += "--parents "
        if show_children:
            cmd += "--children "

        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)
        return result

    def set_template(
        self, cmd_env, proj: str, name: str, body: Optional[str] = None, description: Optional[str] = None
    ) -> Result:
        cmd = self._base_cmd + f"--project '{proj}' template set '{name}' "
        filename = None
        if body:
            filename = "temp-set-template-body.txt"
            self.write_file(filename, body)
            cmd += f"-b '{filename}' "
        if description:
            cmd += f"-d '{description}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

        if filename:
            self.delete_file(filename)
        return result

    def delete_template(self, cmd_env, proj: str, name: str):
        cmd = self._base_cmd + f"--project '{proj}' template del -y '{name}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertResultSuccess(result)

    def current_username(self, cmd_env) -> str:
        result = self.run_cli(cmd_env, self._base_cmd + "config current -f json")
        self.assertResultSuccess(result)
        properties = eval(result.out()).get("profile")
        entry = find_by_prop(properties, "Parameter", "User")[0]
        return entry.get("Value")

    # creates a new user and returns the API key
    def add_user(self, cmd_env, user_name: str, role: str = "contrib") -> str:
        result = self.run_cli(cmd_env, self._base_cmd + f"user set '{user_name}' --role '{role}'")
        self.assertResultSuccess(result)
        self.assertIn("Created service account", result.out())
        if len(result.stdout) > 1:
            # the api token is the second line
            api_token = result.stdout[1]
            return api_token
        return None

    def delete_user(self, cmd_env, user_name):
        result = self.run_cli(cmd_env, self._base_cmd + f"user delete '{user_name}' -y")
        self.assertResultSuccess(result)

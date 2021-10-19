import dataclasses
import os
import shlex
import subprocess
import unittest
from copy import deepcopy
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Optional, Dict


# These are environment variable names used by the application
CT_API_KEY = "CLOUDTRUTH_API_KEY"
CT_ENV = "CLOUDTRUTH_ENVIRONMENT"
CT_PROFILE = "CLOUDTRUTH_PROFILE"
CT_PROJ = "CLOUDTRUTH_PROJECT"
CT_URL = "CLOUDTRUTH_SERVER_URL"
CT_TIMEOUT = "CLOUDTRUTH_REQUEST_TIMEOUT"
CT_REST_DEBUG = "CLOUDTRUTH_REST_DEBUG"

DEFAULT_SERVER_URL = "https://api.cloudtruth.io"
DEFAULT_ENV_NAME = "default"
DEFAULT_PROFILE_NAME = "default"

AUTO_DESCRIPTION = "Automated testing via live_test"

CT_TEST_LOG_COMMANDS = "CT_LIVE_TEST_LOG_COMMANDS"
CT_TEST_LOG_OUTPUT = "CT_LIVE_TEST_LOG_OUTPUT"
CT_TEST_JOB_ID = "CT_LIVE_TEST_JOB_ID"

SRC_ENV = "shell"
SRC_ARG = "argument"
SRC_PROFILE = "profile"
SRC_DEFAULT = "default"

REDACTED = "*****"
DEFAULT_PARAM_VALUE = "-"

# properties
PROP_CREATED = "Created At"
PROP_MODIFIED = "Modified At"
PROP_NAME = "Name"
PROP_TYPE = "Type"
PROP_VALUE = "Value"
PROP_RAW = "Raw"


def get_cli_base_cmd() -> str:
    """
    This is a separate function that does not reference the `self._base_cmd' so it can be called
    during __init__(). It returns the path to the executable (presumably) with the trailing
    space to allow for easier consumption.
    """
    # walk back up looking for top of projects, and goto `target/debug/cloudtruth`
    curr = Path(__file__).absolute()
    subdir = Path("target") / "debug"
    match = False
    while not match and curr:
        possible = curr.parent / subdir
        match = possible.exists()
        curr = curr.parent

    if not match:
        return "cloudtruth "

    for fname in ("cloudtruth", "cloudtruth.exe"):
        file = possible / fname
        if file.exists():
            return str(file) + " "

    # this is a little odd... no executable found in "local" directories
    return "cloudtruth "


@dataclasses.dataclass
class Result:
    return_value: int = 0,
    stdout: List = dataclasses.field(default_factory=list),
    stderr: List = dataclasses.field(default_factory=list),
    timediff: timedelta = timedelta(0)

    def out(self) -> str:
        return "\n".join(self.stdout)

    def err(self) -> str:
        return "\n".join(self.stderr)


class TestCase(unittest.TestCase):
    """
    This extends the unittest.TestCase to add some basic functions
    """
    def __init__(self, *args, **kwargs):
        self._base_cmd = get_cli_base_cmd()
        self.log_commands = int(os.environ.get(CT_TEST_LOG_COMMANDS, "0"))
        self.log_output = int(os.environ.get(CT_TEST_LOG_OUTPUT, "0"))
        self.job_id = os.environ.get(CT_TEST_JOB_ID, "")
        self._projects = None
        self._environments = None
        self._filenames = None
        super().__init__(*args, **kwargs)
        self.maxDiff = None

    def setUp(self) -> None:
        # start each test with empty sets for projects and environments
        self._projects = list()
        self._environments = list()
        self._filenames = set()
        super().setUp()

    def tearDown(self) -> None:
        # tear down any possibly lingering projects -- they should have been deleted in reverse
        # order in case there are any children.
        for proj in reversed(self._projects):
            cmd = self._base_cmd + f"proj del \"{proj}\" --confirm"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # tear down any possibly lingering environments -- they should have been deleted in reverse
        # order in case there are any children.
        for env in reversed(self._environments):
            cmd = self._base_cmd + f"env del \"{env}\" --confirm"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # remove any added files
        for fname in self._filenames:
            os.remove(fname)

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
        Adds the JOB_ID to the name, so multiple tests can run simultaneously.
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
        return deepcopy(os.environ)

    def get_display_env_command(self) -> str:
        if os.name == "nt":
            return "SET"
        return "printenv"

    def assertResultSuccess(self, result: Result):
        """
        This is a convenience method to check the return code, and error output.
        """
        # check the error message is empty first, since it gives the most info about a failure
        self.assertEqual(result.err(), "")
        self.assertEqual(result.return_value, 0)

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

    def run_cli(self, env: Dict[str, str], cmd) -> Result:
        # WARNING: DOS prompt does not like the single quotes, so use double
        cmd = cmd.replace("'", "\"")

        if self.log_commands:
            print(cmd)

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

        start = datetime.now()
        process = subprocess.run(
            cmd, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        delta = datetime.now() - start
        result = Result(
            return_value=process.returncode,
            stdout=process.stdout.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
            stderr=process.stderr.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
            timediff=delta,
        )

        if self.log_output:
            if result.stdout:
                print("\n".join(result.stdout))
            if result.stderr:
                print("\n".join(result.stderr))

        return result

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
            self,
            cmd_env,
            proj: str,
            name: str,
            body: Optional[str] = None,
            description: Optional[str] = None
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

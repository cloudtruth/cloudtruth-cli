import dataclasses
import os
import shlex
import subprocess
import unittest
from copy import deepcopy
from pathlib import Path
from typing import List, Optional, Dict


# These are environment variable names used by the application
CT_API_KEY = "CLOUDTRUTH_API_KEY"
CT_ENV = "CLOUDTRUTH_ENVIRONMENT"
CT_PROFILE = "CLOUDTRUTH_PROFILE"
CT_PROJ = "CLOUDTRUTH_PROJECT"
CT_URL = "CLOUDTRUTH_SERVER_URL"
CT_TIMEOUT = "CLOUDTRUTH_REQUEST_TIMEOUT"

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

    @staticmethod
    def _first_line_contains(stream: List[str], value: str) -> Optional[str]:
        for line in stream:
            if value in line:
                return line
        return None

    def _contains_value(self, stream: List[str], value: str) -> bool:
        return self._first_line_contains(stream, value) is not None

    def _contains_both(self, stream: List[str], one: str, two: str) -> bool:
        line = self._first_line_contains(stream, one)
        if line:
            return two in line
        return False

    @staticmethod
    def _equals(stream: List[str], value: str) -> bool:
        total = "\n".join(stream)
        return total == value

    def out_contains_both(self, one: str, two: str) -> bool:
        return self._contains_both(self.stdout, one, two)

    def out_contains_value(self, one: str) -> bool:
        return self._contains_value(self.stdout, one)

    def out(self) -> str:
        return "\n".join(self.stdout)

    def err_contains_value(self, one: str) -> bool:
        return self._contains_value(self.stderr, one)

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
        super().__init__(*args, **kwargs)
        self.maxDiff = None

    def setUp(self) -> None:
        # start each test with empty sets for projects and environments
        self._projects = set()
        self._environments = set()
        super().setUp()

    def tearDown(self) -> None:
        # tear down any possibly lingering projects -- they should have been deleted
        for proj in self._projects:
            cmd = self._base_cmd + f"proj del \"{proj}\" --confirm"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # tear down any possibly lingering environments -- they should have been deleted
        for env in self._environments:
            cmd = self._base_cmd + f"env del \"{env}\" --confirm"
            subprocess.run(cmd, shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        super().tearDown()

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
                if env_name:
                    self._environments.add(env_name)
                env_name = _next_part(args, "--rename") or _next_part(args, "-r")
                if env_name:
                    self._environments.add(env_name)
            # if we're using any of our 'projects' aliases
            elif set(args) & set(["projects", "project", "proj"]):
                proj_name = _next_part(args, "set")
                if proj_name:
                    self._projects.add(proj_name)
                proj_name = _next_part(args, "--rename") or _next_part(args, "-r")
                if proj_name:
                    self._projects.add(proj_name)

        process = subprocess.run(
            cmd, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        result = Result(
            return_value=process.returncode,
            stdout=process.stdout.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
            stderr=process.stderr.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
        )

        if self.log_output:
            if result.stdout:
                print("\n".join(result.stdout))
            if result.stderr:
                print("\n".join(result.stderr))

        return result

    def get_profile(self, cmd_env, prof_name: str) -> Optional[Dict]:
        result = self.run_cli(cmd_env, self._base_cmd + "config list --values --format csv -s")
        self.assertEqual(result.return_value, 0)
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

    def create_project(self, cmd_env, proj_name: str) -> None:
        result = self.run_cli(cmd_env,
                              self._base_cmd + f"proj set '{proj_name}' -d '{AUTO_DESCRIPTION}'")
        self.assertEqual(result.return_value, 0)

    def delete_project(self, cmd_env, proj_name: str) -> None:
        result = self.run_cli(cmd_env, self._base_cmd + f"proj delete '{proj_name}' --confirm")
        self.assertEqual(result.return_value, 0)

    def create_environment(self, cmd_env, env_name: str, parent: Optional[str] = None) -> None:
        cmd = self._base_cmd + f"env set '{env_name}' "
        if parent:
            cmd += f"-p '{parent}' "
        cmd += f"-d '{AUTO_DESCRIPTION}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertEqual(result.return_value, 0)

    def delete_environment(self, cmd_env, env_name: str) -> None:
        result = self.run_cli(cmd_env, self._base_cmd + f"env del '{env_name}' --confirm")
        self.assertEqual(result.return_value, 0)

    def set_param(
            self,
            cmd_env,
            proj: str,
            name: str,
            value: str,
            secret: Optional[bool] = None,
            env: Optional[str] = None,
            desc: Optional[str] = None,
    ) -> None:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += f"param set '{name}' --value '{value}' "
        if secret is not None:
            cmd += f"--secret '{str(secret).lower()}' "
        if desc:
            cmd += f"--desc '{desc}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertEqual(result.return_value, 0)

    def unset_param(
        self,
        cmd_env,
        proj: str,
        name: str,
        env: Optional[str] = None,
    ):
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += f"param unset '{name}' "
        result = self.run_cli(cmd_env, cmd)
        self.assertEqual(result.return_value, 0)

    def delete_param(self, cmd_env, proj: str, name: str, env: Optional[str] = None) -> None:
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += f"param delete '{name}'"
        result = self.run_cli(cmd_env, cmd)
        self.assertEqual(result.return_value, 0)

    def verify_param(
            self,
            cmd_env,
            proj: str,
            name: str,
            value: str,
            secret: Optional[bool] = None,
            env: Optional[str] = None,
            desc: Optional[str] = None):
        cmd = self._base_cmd + f"--project '{proj}' "
        if env:
            cmd += f"--env '{env}' "
        cmd += "param "

        # check the 'get' output
        result = self.run_cli(cmd_env, cmd + f"get '{name}'")
        self.assertIn(value, result.out())

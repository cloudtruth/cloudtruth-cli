import dataclasses
import os
import subprocess
import unittest
from pathlib import Path
from typing import List, Optional, Dict


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
        self._base_cmd = self.get_cli_base_cmd()
        self.log_commands = 1
        self.log_output = 0
        super().__init__(*args, **kwargs)

    def get_cli_base_cmd(self) -> str:
        """
        Finds where to get the executable image from.
        The result includes an extra space, and whatever other args may be necessary (e.g. api_key)
        """
        if os.environ.get("CI"):
            return "cloudtruth "

        # walk back up looking for top of projects, and goto `target/debug/cloudtruth`
        curr = Path(__file__)
        subdir = Path("target") / "debug"
        match = False
        while not match and curr:
            possible = curr.parent / subdir
            match = possible.exists()
            curr = curr.parent

        if not match:
            return "cloudtruth "

        # TODO: make this more accomodating of other platforms
        return str(possible / "cloudtruth ")

    def get_cmd_env(self):
        return os.environ

    def run_cli(self, env: Dict[str, str], cmd) -> Result:
        if self.log_commands:
            print(cmd)

        process = subprocess.run(
            cmd, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        result = Result(
            return_value=process.returncode,
            stdout=process.stdout.decode("utf-8").split("\n"),
            stderr=process.stderr.decode("utf-8").split("\n"),
        )

        if self.log_output:
            if result.stdout:
                print("\n".join(result.stdout))
            if result.stderr:
                print("\n".join(result.stderr))

        return result

    def create_project(self, cmd_env, proj_name: str) -> None:
        result = self.run_cli(cmd_env, self._base_cmd + f"proj set {proj_name}")
        self.assertEqual(result.return_value, 0)

    def delete_project(self, cmd_env, proj_name: str) -> None:
        result = self.run_cli(cmd_env, self._base_cmd + f" proj delete {proj_name} --confirm")
        self.assertEqual(result.return_value, 0)

    def create_environment(self, cmd_env, env_name: str) -> None:
        result = self.run_cli(cmd_env, self._base_cmd + f"env set {env_name}")
        self.assertEqual(result.return_value, 0)

    def delete_environment(self, cmd_env, env_name: str) -> None:
        result = self.run_cli(cmd_env, self._base_cmd + f"env del {env_name} --confirm")
        self.assertEqual(result.return_value, 0)

    def set_param(self, cmd_env, proj: str, name: str, value: str, secret: bool = False):
        result = self.run_cli(cmd_env, self._base_cmd + f"--project {proj} param set {name} --value {value} --secret {str(secret).lower()}")
        self.assertEqual(result.return_value, 0)

    def delete_param(self, cmd_env, proj: str, name: str):
        result = self.run_cli(cmd_env, self._base_cmd + f"--project {proj} param delete {name}")
        self.assertEqual(result.return_value, 0)


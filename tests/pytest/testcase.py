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

    def out_equals(self, value: str) -> bool:
        return self._equals(self.stdout, value)

    def err_contains_value(self, one: str) -> bool:
        return self._contains_value(self.stderr, one)


class TestCase(unittest.TestCase):
    """
    This extends the unittest.TestCase to add some basic functions
    """
    LOG_COMMANDS = 1
    LOG_OUTPUT = 0

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
        if self.LOG_COMMANDS:
            print(cmd)

        process = subprocess.run(
            cmd, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        result = Result(
            return_value=process.returncode,
            stdout=process.stdout.decode("utf-8").split("\n"),
            stderr=process.stderr.decode("utf-8").split("\n"),
        )

        if self.LOG_OUTPUT:
            if result.stdout:
                print("\n".join(result.stdout))
            if result.stderr:
                print("\n".join(result.stderr))

        return result


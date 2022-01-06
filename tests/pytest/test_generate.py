import dataclasses
from copy import deepcopy
from typing import Optional

from testcase import TestCase
from testcase import CT_REST_DEBUG

DEFAULT_LEN = 15
LOWEST_LEN = 8
MAXIMUM_LEN = 4095


@dataclasses.dataclass
class PasswordFlagTest:
    flag: str
    query: Optional[str] = None
    # TODO: add positive regex here?

    def query_param(self):
        return self.query or "require_" + self.flag


class TestGeneration(TestCase):
    def _get_password(self, cmd_env, pass_cmd: str) -> str:
        result = self.run_cli(cmd_env, pass_cmd)
        self.assertResultSuccess(result)
        return result.out().rstrip()

    def test_generate_password_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        pass_cmd = base_cmd + "generate pw "

        pw = self._get_password(cmd_env, pass_cmd)
        self.assertEqual(DEFAULT_LEN, len(pw))

        # test a variety of lengths
        for length in [LOWEST_LEN, DEFAULT_LEN, 5 * DEFAULT_LEN, MAXIMUM_LEN]:
            pw = self._get_password(cmd_env, pass_cmd + f"--length {length}")
            self.assertEqual(len(pw), length)

        # test the various flags in the request
        debug_env = deepcopy(cmd_env)
        debug_env[CT_REST_DEBUG] = "true"
        flag_tests = [
            PasswordFlagTest(flag="lowercase"),
            PasswordFlagTest(flag="uppercase"),
            PasswordFlagTest(flag="number", query="require_numbers"),
            PasswordFlagTest(flag="symbol", query="require_symbols"),
            PasswordFlagTest(flag="space", query="require_spaces"),
            PasswordFlagTest(flag="hardware", query="require_hardware_generation")
        ]
        for item in flag_tests:
            cmd = pass_cmd
            result = self.run_cli(debug_env, cmd)
            self.assertResultSuccess(result)
            request = result.stdout[0]
            self.assertNotIn(item.query_param(), request)

            cmd = pass_cmd + f"--{item.flag}"
            result = self.run_cli(debug_env, cmd)
            self.assertResultSuccess(result)
            request = result.stdout[0]
            self.assertIn(f"{item.query_param()}=true", request)

            cmd = pass_cmd + f"--no-{item.flag}"
            result = self.run_cli(debug_env, cmd)
            self.assertResultSuccess(result)
            request = result.stdout[0]
            self.assertIn(f"{item.query_param()}=false", request)

        # test with all the flags set, we get the queryparam=true of everything
        s = " --"
        n = " --no-"
        cmd = pass_cmd + s + s.join([i.flag for i in flag_tests]) + n + n.join([i.flag for i in flag_tests])
        result = self.run_cli(debug_env, cmd)
        self.assertResultSuccess(result)
        request = result.stdout[0]
        self.assertTrue(all([f"{i.query_param()}=true" in request for i in flag_tests]))

        # negative testing
        result = self.run_cli(cmd_env, pass_cmd + f"--length {LOWEST_LEN - 1}")
        self.assertResultError(result, f"Password must be {LOWEST_LEN} or more characters")

        result = self.run_cli(cmd_env, pass_cmd + f"--length {MAXIMUM_LEN + 1}")
        self.assertResultError(result, f"Password must be less than {MAXIMUM_LEN + 1} character")

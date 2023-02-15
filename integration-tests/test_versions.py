from testcase import TestCase
from unittest import skipIf
import platform


class TestVersions(TestCase):
    @skipIf(
        platform.system() == "Darwin",
        "Fails to overwrite an existing installed executable without root")
    def test_version_install(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        result = self.run_cli(cmd_env, base_cmd + "--version")
        self.assertResultSuccess(result)
        cli_ver = result.out().split(" ")[-1].strip()

        result = self.run_cli(cmd_env, base_cmd + "version check")
        self.assertResultIn(result, cli_ver)

        result = self.run_cli(cmd_env, base_cmd + "version install -f")
        # self.assertResultUnknown(result)

        result = self.run_cli(cmd_env, base_cmd + "ve get")
        self.assertResultSuccess(result)
        self.assertIn(f"Current CLI version {cli_ver}", result.out())

        result = self.run_cli(cmd_env, base_cmd + "v get --latest")
        self.assertResultSuccess(result)
        self.assertIn("Latest CLI version", result.out())

import platform
import unittest

from testcase import TestCase


class TestMiscellaneous(TestCase):
    @unittest.skipIf(platform.system() == "Windows", "YAML package not found on Windows")
    def test_misc_schema(self):
        # import late, so we can skip on Windows
        from ruamel.yaml import YAML
        import json

        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()
        yaml = YAML()

        result = self.run_cli(cmd_env, base_cmd + "schema")
        self.assertResultSuccess(result)
        default_schema = yaml.load(result.out())

        # specify the output type -- yaml
        result = self.run_cli(cmd_env, base_cmd + "schema --format yaml")
        self.assertResultSuccess(result)
        yaml_schema = yaml.load(result.out())

        # specify the output type -- json
        result = self.run_cli(cmd_env, base_cmd + "schema -f json")
        self.assertResultSuccess(result)
        json_schema = json.loads(result.out())

        # check equivalence of the different methods
        self.assertEqual(default_schema, yaml_schema)
        self.assertEqual(default_schema, json_schema)

        # check the version
        result = self.run_cli(cmd_env, base_cmd + "schema --version")
        self.assertResultSuccess(result)
        version = result.out().strip()
        self.assertEqual(version, default_schema["info"]["version"])

        # check local
        result = self.run_cli(cmd_env, base_cmd + "schema --local")
        self.assertResultSuccess(result)
        local_schema = yaml.load(result.out())

        # check the local version
        result = self.run_cli(cmd_env, base_cmd + "schema --local --version")
        self.assertResultSuccess(result)
        version = result.out().strip()
        self.assertEqual(version, local_schema["info"]["version"])

    def test_misc_completions(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        # just check all the shells generate something
        for shell in ["zsh", "bash", "fish", "powershell", "elvish"]:
            result = self.run_cli(cmd_env, base_cmd + f"completions {shell}")
            self.assertResultSuccess(result)

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

        result = self.run_cli(cmd_env, base_cmd + "schema server")
        self.assertResultSuccess(result)
        default_text = result.out()
        default_schema = yaml.load(result.out())

        # specify the output type -- yaml
        result = self.run_cli(cmd_env, base_cmd + "schema server --format yaml")
        self.assertResultSuccess(result)
        yaml_text = result.out()
        yaml_schema = yaml.load(result.out())

        # specify the output type -- json
        result = self.run_cli(cmd_env, base_cmd + "schema server -f json")
        self.assertResultSuccess(result)
        json_schema = json.loads(result.out())

        # check equivalence of the different methods
        self.assertEqual(default_text, yaml_text)  # pre-evaluation should be same
        self.assertEqual(default_schema, yaml_schema)
        self.assertEqual(default_schema, json_schema)

        # check the version
        result = self.run_cli(cmd_env, base_cmd + "schema server --version")
        self.assertResultSuccess(result)
        version = result.out().strip()
        self.assertEqual(version, default_schema["info"]["version"])

        # check local
        result = self.run_cli(cmd_env, base_cmd + "schema local")
        self.assertResultSuccess(result)
        local_text = result.out()
        local_schema = yaml.load(result.out())

        # check local using yaml
        result = self.run_cli(cmd_env, base_cmd + "schema local -f yaml")
        self.assertResultSuccess(result)
        local_yaml_text = result.out()
        local_yaml = yaml.load(result.out())

        # check local using json
        result = self.run_cli(cmd_env, base_cmd + "schema local --format json")
        self.assertResultSuccess(result)
        local_json = json.loads(result.out())

        # check equivalence of the different methods
        self.assertEqual(local_schema, local_yaml)
        self.assertEqual(local_schema, local_json)
        self.assertEqual(local_text, local_yaml_text)

        # check the local version
        result = self.run_cli(cmd_env, base_cmd + "schema local --version")
        self.assertResultSuccess(result)
        version = result.out().strip()
        self.assertEqual(version, local_schema["info"]["version"])

        # check the diff -- nothing really to compare, just that options succeed
        result = self.run_cli(cmd_env, base_cmd + "schema diff")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, base_cmd + "schema diff --format json")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, base_cmd + "schema diff --format yaml")
        self.assertResultSuccess(result)

    def test_misc_completions(self):
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()

        # just check all the shells generate something
        for shell in ["zsh", "bash", "fish", "powershell", "elvish"]:
            result = self.run_cli(cmd_env, base_cmd + f"completions {shell}")
            self.assertResultSuccess(result)

    @unittest.skipIf(platform.system() == "Windows", "YWindows test issue")
    def test_misc_install(self):
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

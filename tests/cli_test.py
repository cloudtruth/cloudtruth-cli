import argparse
import dataclasses
import subprocess
import sys
import os
from typing import List, Optional, Dict

DEFAULT_SERVER_URL = "https://api.cloudtruth.com/graphql"


def parse_args(*args) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the CloudTruth CLI tests"
    )
    parser.add_argument(
        "-k",
        "--api_key",
        type=str,
        help="CloudTruth API key for server GraphQL authorization",
    )
    parser.add_argument(
        "-u",
        "--url",
        type=str,
        help="CloudTruth server URL",
        default=DEFAULT_SERVER_URL,
    )
    parser.add_argument(
        "--profile",

    )
    return parser.parse_args(*args)

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

    def _contains_value(self, stream: List[str], value):
        return self._first_line_contains(stream, value) is not None

    def _contains_key_value(self, stream: List[str], key: str, value: str) -> bool:
        line = self._first_line_contains(stream, key)
        if line:
            return value in line
        return False

    def out_contains_key_value(self, key: str, value: str) -> bool:
        return self._contains_key_value(self.stdout, key, value)

    def out_contains_value(self, value: str) -> bool:
        return self._contains_value(self.stdout, value)


def run(env: Dict[str, str], args: List[str]) -> Result:
    print(f"Running: {' '.join(args)}")
    process = subprocess.run(
        args, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    return Result(
        return_value=process.returncode,
        stdout=process.stdout.decode("utf-8").split("\n"),
        stderr=process.stderr.decode("utf-8").split("\n"),
    )


def test_environment_crud(env: Dict[str, str], base_args: List[str]):
    # verify `new_name` does not yet exist
    new_name = "test-env-name"
    sub_args = base_args + ["environments"]
    result = run(env, sub_args + ["ls", "-v"])
    assert result.return_value == 0, "Initial environment list failed"
    assert not result.out_contains_key_value(new_name), "Environment pre-exists"

    # create with a description
    orig_desc = "Description on create"
    result = run(env, sub_args + ["set", {new_name},  "--desc", {orig_desc}])
    assert result.return_value == 0, "Create command failed"
    result = run(env, sub_args + ["ls", "-v"])
    assert result.return_value == 0, "Post-create environment list failed"
    assert result.out_contains_key_value(new_name, orig_desc), "Created environment not in list"

    # update the description
    new_desc = "Updated description"
    result = run(env, sub_args + ["set", new_name, "--desc", new_desc])
    assert result.return_value == 0, "Update environment description"
    result = run(env, sub_args + ["ls", "-v"])
    assert result.return_value == 0, "Post-update environment list failed"
    assert result.out_contains_key_value(new_name, new_desc), "Updated environment not in list"

    # test the list without the table
    result = run(env, sub_args + ["list"])
    assert result.return_value == 0, "Environment list without values failed"
    assert result.out_contains_value(new_name), "Environment not in list"
    assert not result.out_contains_key_value(new_name, new_desc), "Short list contains value"

    # delete the description
    result = run(env, sub_args + ["delete", new_name, "--confirm"])
    assert result.return_value == 0, "Delete (With confirm)"
    result = run(env, sub_args + ["ls", "-v"])
    assert result.return_value == 0, "Environment list failed"
    assert result.out_contains_key_value(new_name, new_desc)


def cli_test(*args):
    result = 0
    args = parse_args(*args)
    if args.url is None:
        args.url = os.environ("CLOUDTRUTH_API_KEY")

    env = os.environ.copy()
    if args.url:
        env["CLOUDTRUTH_SERVER_URL"] = args.url
    if args.api_key:
        env["CLOUDTRUTH_API_KEY"] = args.api_key

    # TODO: figure out right way to get this
    base_args = ["/Users/rick/cloudtruth-cli/target/debug/cloudtruth"]

    # TODO: create loop and add try/catch
    # TODO: find functions matching name, so do not need to add call to function
    try:
        test_environment_crud(env, base_args)
    except AssertionError as err:
        breakpoint()
        result = 1

    return result

if __name__ == "__main__":
    sys.exit(cli_test(sys.argv[1:]))

import argparse
import dataclasses
import subprocess
import sys
import os
from typing import List, Optional, Dict

DEFAULT_SERVER_URL = "https://api.cloudtruth.com/graphql"
LOG_COMMANDS = 1
LOG_OUTPUT = 1


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


def run(env: Dict[str, str], cmd) -> Result:
    if LOG_COMMANDS:
        print(cmd)

    process = subprocess.run(
        cmd, env=env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    result = Result(
        return_value=process.returncode,
        stdout=process.stdout.decode("utf-8").split("\n"),
        stderr=process.stderr.decode("utf-8").split("\n"),
    )

    if LOG_OUTPUT:
        if result.stdout:
            print("\n".join(result.stdout))
        if result.stderr:
            print("\n".join(result.stderr))

    return result


def test_environment_crud(env: Dict[str, str], base_cmd: str):
    # verify `env_name` does not yet exist
    env_name = "test-env-name"
    sub_cmd = base_cmd + " environments "
    result = run(env, sub_cmd + "ls -v")
    assert result.return_value == 0, "Initial environment list failed"
    assert not result.out_contains_value(env_name), "Environment pre-exists"

    # create with a description
    orig_desc = "Description on create"
    result = run(env, sub_cmd + f"set {env_name} --desc \"{orig_desc}\"")
    assert result.return_value == 0, "Create command failed"
    result = run(env, sub_cmd + "ls -v")
    assert result.return_value == 0, "Post-create environment list failed"
    assert result.out_contains_both(env_name, orig_desc), "Created environment not in list"

    # update the description
    new_desc = "Updated description"
    result = run(env, sub_cmd + f"set {env_name} --desc \"{new_desc}\"")
    assert result.return_value == 0, "Update environment description"
    result = run(env, sub_cmd + "ls --values")
    assert result.return_value == 0, "Post-update environment list failed"
    assert result.out_contains_both(env_name, new_desc), "Updated environment not in list"

    # test the list without the table
    result = run(env, sub_cmd + "list")
    assert result.return_value == 0, "Environment list without values failed"
    assert result.out_contains_value(env_name), "Environment not in list"
    assert not result.out_contains_both(env_name, new_desc), "Short list contains value"

    # delete the description
    result = run(env, sub_cmd + f"delete {env_name} --confirm")
    assert result.return_value == 0, "Delete (With confirm)"
    result = run(env, sub_cmd + "ls -v")
    assert result.return_value == 0, "Environment list failed"
    assert not result.out_contains_value(env_name), "Environment deleted"

    # do it again, see we have success and a warning
    result = run(env, sub_cmd + f"delete {env_name} --confirm")
    assert result.return_value == 0, "Delete #2"
    assert result.err_contains_value(f"Environment '{env_name}' does not exist"), "Re-delete warn"


def test_project_crud(env: Dict[str, str], base_cmd: str):
    # verify `proj_name` does not yet exist
    proj_name = "test-proj-name"
    sub_cmd = base_cmd + " projects "
    result = run(env, sub_cmd + "ls -v")
    assert result.return_value == 0, "Initial project list failed"
    assert not result.out_contains_value(proj_name), "Project pre-exists"

    # create with a description
    orig_desc = "Description on create"
    result = run(env, sub_cmd + f"set {proj_name} --desc \"{orig_desc}\"")
    assert result.return_value == 0, "Create command failed"
    result = run(env, sub_cmd + "ls -v")
    assert result.return_value == 0, "Post-create project list failed"
    assert result.out_contains_both(proj_name, orig_desc), "Created project not in list"

    # update the description
    new_desc = "Updated description"
    result = run(env, sub_cmd + f"set {proj_name} --desc \"{new_desc}\"")
    assert result.return_value == 0, "Update project description"
    result = run(env, sub_cmd + "ls --values")
    assert result.return_value == 0, "Post-update project list failed"
    assert result.out_contains_both(proj_name, new_desc), "Updated project not in list"

    # test the list without the table
    result = run(env, sub_cmd + "list")
    assert result.return_value == 0, "Project list without values failed"
    assert result.out_contains_value(proj_name), "Project not in list"
    assert not result.out_contains_both(proj_name, new_desc), "Short list contains value"

    # delete the description
    result = run(env, sub_cmd + f"delete {proj_name} --confirm")
    assert result.return_value == 0, "Delete (With confirm)"
    result = run(env, sub_cmd + "ls -v")
    assert result.return_value == 0, "Project list failed"
    assert not result.out_contains_value(proj_name), "Project deleted"

    # do it again, see we have success and a warning
    result = run(env, sub_cmd + f"delete {proj_name} --confirm")
    assert result.return_value == 0, "Delete #2"
    assert result.err_contains_value(f"Project '{proj_name}' does not exist"), "Re-delete warn"


def test_parameters_basic_crud(env: Dict[str, str], base_cmd: str):
    # add a new project
    proj_name = "test-param-basic"
    result = run(env, base_cmd + f" projects set {proj_name} -d 'test_parameters_basic_crud() test'")
    assert result.return_value == 0, "Project create"

    # check that there are no parameters
    sub_cmd = base_cmd + f" --project {proj_name} parameters "
    result = run(env, sub_cmd + "list")
    assert result.return_value == 0, "Initial empty parameters"
    assert result.out_contains_value("There are no parameters in your account"), "Project empty"

    # same result with the --values flag
    result = run(env, sub_cmd + "list --values")
    assert result.return_value == 0, "Initial empty parameters"
    assert result.out_contains_value("No CloudTruth variables found!"), "Project --values empty"

    # same result with the --values and --secrets flag
    result = run(env, sub_cmd + "list --values --secrets")
    assert result.return_value == 0, "Initial empty parameters"
    assert result.out_contains_value("No CloudTruth variables found!"), "Project --secrets empty"

    ########
    # add first, non-secret parameter
    key1 = "my_param"
    value1 = "cRaZy value"
    desc1 = "this is just a test description"
    result = run(env, sub_cmd + f"set {key1} --value \"{value1}\" --desc \"{desc1}\"")
    assert result.return_value == 0, "First variable create"

    result = run(env, sub_cmd + f"ls -v")
    assert result.out_contains_both(key1, value1)
    assert result.out_contains_both(key1, desc1)
    assert result.out_equals("""\
+----------+-------------+---------+---------------------------------+
| Name     | Value       | Source  | Description                     |
+----------+-------------+---------+---------------------------------+
| my_param | cRaZy value | default | this is just a test description |
+----------+-------------+---------+---------------------------------+
""")

    # delete the parameter
    result = run(env, sub_cmd + f"delete {key1}")
    assert result.return_value == 0, "Delete param"

    # make sure it is gone
    result = run(env, sub_cmd + "list --values --secrets")
    assert result.return_value == 0, "Initial empty parameters"
    assert result.out_contains_value("No CloudTruth variables found!"), "Project again empty"

    # delete the project
    result = run(env, base_cmd + f" projects delete {proj_name} --confirm")
    assert result.return_value == 0, "Project delete"


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
    base_cmd = "target/debug/cloudtruth"

    # TODO: find functions matching name, so do not need to add call to function
    tests = [
        test_environment_crud,
        test_project_crud,
        test_parameters_basic_crud,
    ]
    for test_fn in tests:
        try:
            test_fn(env, base_cmd)
        except AssertionError as err:
            print(f"ERROR: {test_fn.__name__}() failed: {err}")
            result = 1

    return result


if __name__ == "__main__":
    sys.exit(cli_test(sys.argv[1:]))

import os

from collections import OrderedDict
from typing import List

from testcase import TestCase
from testcase import CT_REST_DEBUG


# some environment variables for test control (without file modification)
CT_PERSIST = "CLOUDTRUTH_TEST_PERSIST"
CT_PARAM_COUNT = "CLOUDTRUTH_TEST_PARAMETER_COUNT"


ENV_RESOLVE = "env-resolve"
PROJ_RESOLVE = "proj-resolve"
DEFAULT_PARAM_COUNT = 20


def parse_time(value: str) -> int:
    """
    Parses the string into a integer representing the number of milliseconds.
    """
    if value.endswith("ms"):
        return int(float(value.replace("ms", "")))
    return int(float(value.replace("s", "")) * 1000)


def parse_timing(timing_info: List[List[int]], lines: List[str]) -> List[List[int]]:
    url_count = 0
    for line in lines:
        if not line.startswith("URL"):
            continue

        elapsed = line.split("elapsed: ")[-1]
        curr_list = timing_info[url_count]
        curr_list.append(parse_time(elapsed))
        timing_info[url_count] = curr_list
        url_count += 1

    return timing_info


def print_timing_info(test_name: str, timing_info: OrderedDict) -> None:
    print("\n" + '=' * 40 + f"  {test_name} " + '=' * 40)
    print("Times in milliseconds")
    for operation, times in timing_info.items():
        pretty = ", ".join([str(x) for x in times])
        print(f"{operation}  ==>  [{pretty}]")


class TestTiming(TestCase):
    def setUp(self) -> None:
        self.leave_up = CT_PERSIST in os.environ
        self.param_count = int(os.environ.get(CT_PARAM_COUNT) or DEFAULT_PARAM_COUNT)
        self.param_prefix = "param"
        super().setUp()

    def _param_name(self, index: int) -> str:
        return f"{self.param_prefix}{index}"

    def _parameter_create_timing(self, proj_name: str, num_values: int, secret: bool) -> OrderedDict:
        base_cmd = self.get_cli_base_cmd()
        get_cmd = base_cmd + f"--project '{proj_name}' param get "
        cmd_env = self.get_cmd_env()
        cmd_env[CT_REST_DEBUG] = "true"
        create_timing = [[], [], [], [], [], ]
        get_timing = [[], [], [], []]

        for index in range(num_values):
            result = self.set_param(cmd_env, proj_name, self._param_name(index), "abc123", secret=secret)
            self.assertEqual(result.return_value, 0)
            create_timing = parse_timing(create_timing, result.stdout)

            result = self.run_cli(cmd_env, get_cmd + f"'{self._param_name(index)}'")
            self.assertEqual(result.return_value, 0)
            get_timing = parse_timing(get_timing, result.stdout)

        rval = OrderedDict()
        rval["create-" + ENV_RESOLVE] = create_timing[0]
        rval["create-" + PROJ_RESOLVE] = create_timing[1]
        rval["create-param-set"] = create_timing[2]
        rval["create-param-set"] = create_timing[3]
        rval["create-value-set"] = create_timing[4]

        rval["get-" + ENV_RESOLVE] = get_timing[0]
        rval["get-" + PROJ_RESOLVE] = get_timing[1]
        rval["get-param-resolve"] = get_timing[2]
        rval["get-param-retrieve"] = get_timing[3]
        return rval

    def _parameter_retrieve_timing(self, proj_name: str, num_values: int) -> OrderedDict:
        base_cmd = self.get_cli_base_cmd()
        get_cmd = base_cmd + f"--project '{proj_name}' param get "
        cmd_env = self.get_cmd_env()
        cmd_env[CT_REST_DEBUG] = "true"

        timing_info = [[], [], [], []]
        for index in range(num_values):
            result = self.run_cli(cmd_env, get_cmd + f"'{self._param_name(index)}'")
            self.assertEqual(result.return_value, 0)
            timing_info = parse_timing(timing_info, result.stdout)

        rval = OrderedDict()
        rval[ENV_RESOLVE] = timing_info[0]
        rval[PROJ_RESOLVE] = timing_info[1]
        rval["param-resolve"] = timing_info[2]
        rval["param-retrieve"] = timing_info[3]
        return rval

    def test_timing_secrets(self) -> None:
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-timing-secrets")
        self.create_project(cmd_env, proj_name)

        timing = self._parameter_create_timing(proj_name, self.param_count, True)
        print_timing_info("timing-secrets", timing)

        # cleanup
        if not self.leave_up:
            self.delete_project(cmd_env, proj_name)

    def test_timing_params(self) -> None:
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-timing-params")
        self.create_project(cmd_env, proj_name)

        timing = self._parameter_create_timing(proj_name, self.param_count, False)
        print_timing_info("timing-parameters", timing)

        # cleanup
        if not self.leave_up:
            self.delete_project(cmd_env, proj_name)

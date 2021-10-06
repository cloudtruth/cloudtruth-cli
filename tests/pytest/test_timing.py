import os

from collections import OrderedDict
from datetime import timedelta
from typing import List

from testcase import TestCase
from testcase import CT_REST_DEBUG


# some environment variables for test control (without file modification)
CT_PERSIST = "CLOUDTRUTH_TEST_PERSIST"
CT_PARAM_COUNT = "CLOUDTRUTH_TEST_PARAMETER_COUNT"
CT_TEMPLATE_COUNT = "CLOUDTRUTH_TEST_TEMPLATE_COUNT"

ENV_RESOLVE = "env-resolve"
PROJ_RESOLVE = "proj-resolve"
DEFAULT_PARAM_COUNT = 10
DEFAULT_TEMPLATE_COUNT = 5


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


def delta_to_msecs(delta: timedelta) -> int:
    """Converts a timedelta into an integer number of milliseconds."""
    return int(delta.seconds * 1000 + delta.microseconds / 1000)


class TestTiming(TestCase):
    def setUp(self) -> None:
        self.leave_up = CT_PERSIST in os.environ
        self.param_count = int(os.environ.get(CT_PARAM_COUNT) or DEFAULT_PARAM_COUNT)
        self.param_prefix = "param"
        self.template_count = int(os.environ.get(CT_TEMPLATE_COUNT) or DEFAULT_TEMPLATE_COUNT)
        super().setUp()

    def _param_name(self, index: int) -> str:
        return f"{self.param_prefix}{index}"

    def _parameter_create_timing(self, proj_name: str, num_values: int, secret: bool) -> OrderedDict:
        base_cmd = self.get_cli_base_cmd()
        get_cmd = base_cmd + f"--project '{proj_name}' param get "
        list_cmd = base_cmd + f"--project '{proj_name}' param list -s -v"
        cmd_env = self.get_cmd_env()
        cmd_env[CT_REST_DEBUG] = "true"
        create_timing = [[], [], [], [], [], ]
        create_total = []
        get_timing = [[], [], [], ]
        get_total = []
        list_timing = [[], [], [], [], ]
        list_total = []

        for index in range(num_values):
            result = self.set_param(cmd_env, proj_name, self._param_name(index), "abc123", secret=secret)
            self.assertResultSuccess(result)
            create_timing = parse_timing(create_timing, result.stdout)
            create_total.append(delta_to_msecs(result.timediff))

            result = self.run_cli(cmd_env, get_cmd + f"'{self._param_name(index)}'")
            self.assertResultSuccess(result)
            get_timing = parse_timing(get_timing, result.stdout)
            get_total.append(delta_to_msecs(result.timediff))

            result = self.run_cli(cmd_env, list_cmd)
            self.assertResultSuccess(result)
            list_timing = parse_timing(list_timing, result.stdout)
            list_total.append(delta_to_msecs(result.timediff))

        rval = OrderedDict()
        rval["create-" + ENV_RESOLVE] = create_timing[0]
        rval["create-" + PROJ_RESOLVE] = create_timing[1]
        rval["create-param-get"] = create_timing[2]
        rval["create-param-set"] = create_timing[3]
        rval["create-value-set"] = create_timing[4]
        rval["create-total"] = create_total

        rval["get-" + ENV_RESOLVE] = get_timing[0]
        rval["get-" + PROJ_RESOLVE] = get_timing[1]
        rval["get-param-retrieve"] = get_timing[2]
        rval["get-total"] = get_total

        rval["list-" + ENV_RESOLVE] = list_timing[0]
        rval["list-" + PROJ_RESOLVE] = list_timing[1]
        rval["list-param-list"] = list_timing[2]
        rval["list-total"] = list_total
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

    def test_timing_template(self) -> None:
        cmd_env = self.get_cmd_env()
        base_cmd = self.get_cli_base_cmd()
        proj_name = self.make_name("test-timing-template")
        self.create_project(cmd_env, proj_name)
        env_name = self.make_name("template-timing-env")
        self.create_environment(cmd_env, env_name)

        proj_cmd = base_cmd + f"--project '{proj_name}' --env '{env_name}' "

        # create a couple parameters
        param1 = "param1"
        value1a = "first value"
        param2 = "param2"
        value2a = "another first"
        self.set_param(cmd_env, proj_name, param1, value=value1a, env=env_name)
        self.set_param(cmd_env, proj_name, param2, value=value2a, env=env_name)

        filename = "temp-timing.txt"
        body = """\
PARAMETER1=PARAM1
PARAMETER2=PARAM2
"""
        raw_body = body.replace("PARAM1", f"{{{{{param1}}}}}").replace("PARAM2", f"{{{{{param2}}}}}")
        self.write_file(filename, raw_body)
        temp_name = "timing-template"
        result = self.run_cli(cmd_env, proj_cmd + f"template set '{temp_name}' -b {filename}")
        self.assertResultSuccess(result)

        # tag the environment
        tag_name = "my-temp-timing-tag"
        result = self.run_cli(cmd_env, base_cmd + f"env tag set '{env_name}' '{tag_name}'")
        self.assertResultSuccess(result)

        # update the values
        value1b = "second value"
        value2b = "first loser"
        self.set_param(cmd_env, proj_name, param1, value=value1b, env=env_name)
        self.set_param(cmd_env, proj_name, param2, value=value2b, env=env_name)

        eval_body_a = body.replace("PARAM1", f"{value1a}").replace("PARAM2", f"{value2a}")
        eval_body_b = body.replace("PARAM1", f"{value1b}").replace("PARAM2", f"{value2b}")

        temp_get_raw_current = []
        temp_get_current = []
        temp_get_tag = []
        temp_get_raw_tag = []

        temp_get = proj_cmd + f"template get '{temp_name}' "

        for _ in range(self.template_count):
            result = self.run_cli(cmd_env, temp_get + f"--as-of '{tag_name}' --raw")
            self.assertResultSuccess(result)
            self.assertEqual(result.out(), raw_body)
            temp_get_raw_current.append(delta_to_msecs(result.timediff))

            result = self.run_cli(cmd_env, temp_get)
            self.assertResultSuccess(result)
            self.assertEqual(result.out(), eval_body_b)
            temp_get_current.append(delta_to_msecs(result.timediff))

            result = self.run_cli(cmd_env, temp_get + f"--as-of '{tag_name}'")
            self.assertResultSuccess(result)
            self.assertEqual(result.out(), eval_body_a)
            temp_get_tag.append(delta_to_msecs(result.timediff))

            result = self.run_cli(cmd_env, temp_get + f"--as-of '{tag_name}' --raw")
            self.assertResultSuccess(result)
            self.assertEqual(result.out(), raw_body)
            temp_get_raw_tag.append(delta_to_msecs(result.timediff))

        timing_info = OrderedDict()
        timing_info["template-get-raw"] = temp_get_raw_current
        timing_info["template-get-raw-tag"] = temp_get_raw_tag
        timing_info["template-get-current"] = temp_get_current
        timing_info["template-get-tag"] = temp_get_tag
        print_timing_info("test_timing_template", timing_info)

        # cleanup
        self.delete_file(filename)
        self.delete_environment(cmd_env, env_name)
        self.delete_project(cmd_env, proj_name)

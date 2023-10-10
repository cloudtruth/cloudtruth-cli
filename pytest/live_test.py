#!/usr/bin/env python3
import argparse
import dataclasses
import inspect
import os
import pdb
import subprocess
import sys
import traceback
import unittest

from datetime import datetime
from pathlib import Path
from typing import Dict
from typing import List
from typing import Optional

from testcase import get_cli_base_cmd
from testcase import CT_API_KEY, CT_URL, CT_PROFILE, CT_REST_DEBUG
from testcase import CT_TEST_JOB_ID, CT_TEST_LOG_COMMANDS, CT_TEST_LOG_OUTPUT
from testcase import CT_TEST_LOG_COMMANDS_ON_FAILURE, CT_TEST_LOG_OUTPUT_ON_FAILURE
from testcase import CT_TEST_KNOWN_ISSUES


# NOTE: these constants are used to determine tag names
ERROR = "error"
FAILURE = "failure"
SKIPPED = "skipped"
SUCCESS = "success"


@dataclasses.dataclass
class TestCaseResults:
    testname: str
    classname: str
    filename: str
    line: int
    result: Optional[str] = None
    message: Optional[str] = None
    starttime: Optional[datetime] = None
    endtime: Optional[datetime] = None


def parse_args(*args) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the CloudTruth CLI tests")
    parser.add_argument("-p", "--profile", type=str, help="CLI profile to use for tests")
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
    )
    parser.add_argument(
        "-v",
        "--verbosity",
        type=int,
        default=3,
        help="Unittest verbosity level",
    )
    parser.add_argument("--rest-debug", dest="rest_debug", action="store_true", help="Enable REST debug logging")
    parser.add_argument("--pdb", dest="pdb", action="store_true", help="Open the debugger when a test fails")

    parser.add_argument("--debug", dest="debug", action="store_true", help="Equivalent of --pdb --failfast")
    parser.add_argument(
        "--file",
        dest="file_filter",
        type=str,
        default="test_*.py",
        help="Filter the files run using the specified pattern",
    )
    parser.add_argument("--failfast", action="store_true", help="Stop the test on first error")
    parser.add_argument(
        "-lc", "--log-commands", dest="log_commands", action="store_true", help="Print the commands to stdout."
    )
    parser.add_argument(
        "-lo", "--log-output", dest="log_output", action="store_true", help="Print the output to stdout."
    )
    parser.add_argument(
        "-la", "--log-all", dest="log_all", action="store_true", help="Print the output and commands to stdout"
    )
    parser.add_argument(
        "-lcf",
        "--log-commands-on-failure",
        dest="log_commands_on_failure",
        action="store_true",
        help="Print the commands to stdout when a test fails",
    )
    parser.add_argument(
        "-lof",
        "--log-output-on-failure",
        dest="log_output_on_failure",
        action="store_true",
        help="Print the output to stdout when a test fails",
    )
    parser.add_argument(
        "-laf",
        "--log-all-on-failure",
        dest="log_all_on_failure",
        action="store_true",
        help="Print the output and commands to stdout when a test fails",
    )
    parser.add_argument(
        "--job-id",
        type=str,
        dest="job_id",
        help="Job Identifier to use as a suffix on project and environment names (default: testcli)",
    )
    parser.add_argument(
        "-f",
        "--filter",
        dest="test_filter",
        nargs="+",
        default=[],
        help="Only include tests containing the provided string(s) in the name",
    )
    parser.add_argument(
        "-l",
        "--list",
        dest="list_only",
        action="store_true",
        help="Only print the tests that will be run (without running them).",
    )
    parser.add_argument("--before", dest="before", help="Only run tests before the specified string")
    parser.add_argument("--after", dest="after", help="Only run tests after the specified string")
    parser.add_argument(
        "--exclude",
        dest="test_exclude",
        nargs="+",
        default=[],
        help="Exclude tests containing the provided string(s) in the name",
    )
    parser.add_argument("-r", "--reports", dest="reports", action="store_true", help="Write summary report information")
    parser.add_argument("--known-issues", dest="known_issues", action="store_true", help="don't skip known issues")
    return parser.parse_args(*args)


def error_message(tb: traceback, ae: AssertionError) -> str:
    return "".join(traceback.format_tb(tb)) + "\n\nAssertion:\n" + str(ae)


def name_from_test(test: unittest.case.TestCase) -> str:
    return test._testMethodName


def debugTestRunner(enable_debug: bool, verbosity: int, failfast: bool):
    """Overload the TextTestRunner to conditionally drop into pdb on an error/failure."""

    class DebugTestResult(unittest.TextTestResult):
        def __init__(self, stream, descriptions, verbosity):
            super().__init__(stream=stream, descriptions=descriptions, verbosity=verbosity)
            self.testCaseData = {}

        def addError(self, test: unittest.case.TestCase, err) -> None:
            # called before tearDown()
            traceback.print_exception(*err)
            if enable_debug:
                pdb.post_mortem(err[2])
            name = name_from_test(test)
            self.testCaseData[name].result = ERROR
            self.testCaseData[name].message = error_message(err[2], err[1])
            super().addError(test, err)

        def addFailure(self, test: unittest.case.TestCase, err) -> None:
            traceback.print_exception(*err)
            if enable_debug:
                pdb.post_mortem(err[2])
            name = name_from_test(test)
            self.testCaseData[name].result = FAILURE
            self.testCaseData[name].message = error_message(err[2], err[1])
            super().addFailure(test, err)

        def addSuccess(self, test: unittest.case.TestCase) -> None:
            name = name_from_test(test)
            self.testCaseData[name].result = SUCCESS
            super().addSuccess(test)

        def addSkip(self, test: unittest.case.TestCase, reason: str) -> None:
            name = name_from_test(test)
            self.testCaseData[name].result = SKIPPED
            self.testCaseData[name].message = reason
            super().addSkip(test, reason)

        def startTest(self, test: unittest.case.TestCase) -> None:
            super().startTest(test)
            topdir = Path(__file__).parent.absolute().as_posix() + "/"
            name = name_from_test(test)
            fullpath = inspect.getsourcefile(type(test))
            _, line = inspect.getsourcelines(getattr(test, name))
            filename = fullpath.replace(topdir, "")
            classname = test.__module__ + "." + test.__class__.__name__
            data = TestCaseResults(name, classname, filename, line, starttime=datetime.now())
            self.testCaseData[name] = data

        def stopTest(self, test: unittest.case.TestCase) -> None:
            super().stopTest(test)
            name = name_from_test(test)
            self.testCaseData[name].endtime = datetime.now()

    return unittest.TextTestRunner(
        verbosity=verbosity, failfast=failfast, resultclass=DebugTestResult, stream=sys.stdout
    )


def count_result(items: List[TestCaseResults], result: str) -> int:
    return len([x for x in items if x.result == result])


def print_props(props: Dict) -> str:
    return " ".join(f"{k}={v}" for k, v in props.items())


def write_reports(results) -> None:
    suites = {}
    for item in results.testCaseData.values():
        name = item.classname
        entries = suites[name] if name in suites else []
        entries.append(item)
        suites[name] = entries

    for classname, testcases in suites.items():
        suite_props = {
            "name": classname,
            "tests": len(testcases),
            "failures": count_result(testcases, FAILURE),
            "errors": count_result(testcases, ERROR),
            "skipped": count_result(testcases, SKIPPED),
            "success": count_result(testcases, SUCCESS),
            "filename": next(iter(testcases)).filename,  # just grab filename from the first item
        }
        print("Suite: " + print_props(suite_props))

        for test in testcases:
            delta = test.endtime - test.starttime
            case_props = {
                "name": test.testname,
                # 'classname': test.classname,
                # 'file': f"{test.filename}:{test.line}",
                "line": test.line,
                "timestamp": test.starttime.isoformat(),
                "time": delta.total_seconds(),
                "status": test.result,
            }
            print("    Case: " + print_props(case_props))


def print_suite(suite):
    if hasattr(suite, "__iter__"):
        for x in suite:
            print_suite(x)
    elif hasattr(suite, "_testMethodName"):
        name = getattr(suite, "_testMethodName")
        print(f"{name}")
    else:
        print("invalid")


def filter_suite(suite, func):
    for testmodule in suite:
        for testsuite in testmodule:
            tests_to_remove = []
            for index, testcase in enumerate(testsuite._tests):
                if func(testcase):
                    tests_to_remove.append(index)

            # do this in reverse order, so index does not change
            for index in reversed(tests_to_remove):
                testsuite._tests.pop(index)
    return suite


def filter_before(suite, before: str):
    def is_before(testcase: str) -> bool:
        return testcase._testMethodName > before

    return filter_suite(suite, is_before)


def filter_after(suite, after: str):
    def is_after(testcase: str) -> bool:
        return testcase._testMethodName < after

    return filter_suite(suite, is_after)


def filter_exclude(suite, exclude: str):
    def is_excluded(testcase: str) -> bool:
        return exclude in testcase._testMethodName

    return filter_suite(suite, is_excluded)


def live_test(*args):
    args = parse_args(*args)
    env = os.environ
    if args.url:
        env[CT_URL] = args.url
    if args.api_key:
        env[CT_API_KEY] = args.api_key
    if args.profile:
        env[CT_PROFILE] = args.profile
    if args.rest_debug:
        env[CT_REST_DEBUG] = "true"
    if args.log_all:
        args.log_commands = True
        args.log_output = True
    if args.log_all_on_failure:
        args.log_commands_on_failure = True
        args.log_output_on_failure = True
    env[CT_TEST_LOG_COMMANDS] = str(int(args.log_commands))
    env[CT_TEST_LOG_OUTPUT] = str(int(args.log_output))
    env[CT_TEST_LOG_COMMANDS_ON_FAILURE] = str(int(args.log_commands_on_failure))
    env[CT_TEST_LOG_OUTPUT_ON_FAILURE] = str(int(args.log_output_on_failure))

    if not args.job_id:
        import uuid

        args.job_id = f"local-{uuid.uuid4()}"
    print(f"JOB_ID: {args.job_id}")
    env[CT_TEST_JOB_ID] = args.job_id

    if args.known_issues:
        env[CT_TEST_KNOWN_ISSUES] = True

    cli = get_cli_base_cmd()
    print(f"CloudTruth command: {cli}")
    subprocess.run(cli + "config current -x", shell=True)

    # propagate the debug flags
    if args.debug:
        args.pdb = True
        args.failfast = True

    test_directory = "."
    loader = unittest.TestLoader()
    applied_filter = []
    if args.file_filter:
        applied_filter.append(f"file: {args.file_filter}")

    if args.test_filter:
        applied_filter.append(f"filters: {', '.join(args.test_filter)}")
        loader.testNamePatterns = [f"*{_}*" for _ in args.test_filter]
    suite = loader.discover(test_directory, pattern=args.file_filter)

    if args.before:
        applied_filter.append(f"before: {args.before}")
        suite = filter_before(suite, args.before)

    if args.after:
        applied_filter.append(f"after: {args.after}")
        suite = filter_after(suite, args.after)

    if args.test_exclude:
        applied_filter.append(f"excludes: {', '.join(args.test_exclude)}")
        for ex in args.test_exclude:
            suite = filter_exclude(suite, ex)

    if suite.countTestCases() == 0:
        # must be because of a filter or file filter
        sep = "\n\t"
        print(f"No tests matching:{sep}{sep.join(applied_filter)}")
        return 3

    if args.list_only:
        print_suite(suite)
        return 0

    runner = debugTestRunner(enable_debug=args.pdb, verbosity=args.verbosity, failfast=args.failfast)
    test_result = runner.run(suite)

    if args.reports:
        write_reports(test_result)

    rval = 0
    if len(test_result.errors):
        rval += 1
    if len(test_result.failures):
        rval += 2
    return rval


if __name__ == "__main__":
    sys.exit(live_test(sys.argv[1:]))

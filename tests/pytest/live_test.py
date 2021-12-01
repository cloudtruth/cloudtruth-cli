import argparse
import os
import pdb
import subprocess
import sys
import traceback
import unittest

from testcase import get_cli_base_cmd
from testcase import CT_API_KEY, CT_URL
from testcase import CT_TEST_JOB_ID, CT_TEST_LOG_COMMANDS, CT_TEST_LOG_OUTPUT


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
    )
    parser.add_argument(
        "-v",
        "--verbosity",
        type=int,
        default=3,
        help="Unittest verbosity level",
    )
    parser.add_argument(
        "--pdb",
        dest="pdb",
        action='store_true',
        help="Open the debugger when a test fails"
    )

    parser.add_argument(
        "--debug",
        dest="debug",
        action='store_true',
        help="Equivalent of --pdb --failfast"
    )
    parser.add_argument(
        "--file",
        dest="file_filter",
        type=str,
        default="test_*.py",
        help="Filter the files run using the specified pattern"
    )
    parser.add_argument(
        "--failfast",
        action="store_true",
        help="Stop the test on first error"
    )
    parser.add_argument(
        "-lc",
        "--log-commands",
        dest="log_commands",
        action="store_true",
        help="Print the commands to stdout."
    )
    parser.add_argument(
        "-lo",
        "--log-output",
        dest="log_output",
        action="store_true",
        help="Print the output to stdout."
    )
    parser.add_argument(
        "--job-id",
        type=str,
        dest="job_id",
        help="Job Identifier to use as a suffix on project and environment names"
    )
    parser.add_argument(
        "--filter",
        dest="test_filter",
        nargs="+",
        default=[],
        help="Only include tests containing the provided string(s) in the name"
    )
    parser.add_argument(
        "--list",
        dest="list_only",
        action="store_true",
        help="Only print the tests that will be run (without running them)."
    )
    parser.add_argument(
        "--before",
        dest="before",
        help="Only run tests before the specified string"
    )
    parser.add_argument(
        "--after",
        dest="after",
        help="Only run tests after the specified string"
    )
    parser.add_argument(
        "--exclude",
        dest="test_exclude",
        nargs="+",
        default=[],
        help="Exclude tests containing the provided string(s) in the name"
    )
    return parser.parse_args(*args)


def debugTestRunner(enable_debug: bool, verbosity: int, failfast: bool):
    """Overload the TextTestRunner to conditionally drop into pdb on an error/failure."""
    class DebugTestResult(unittest.TextTestResult):
        def addError(self, test, err):
            # called before tearDown()
            traceback.print_exception(*err)
            if enable_debug:
                pdb.post_mortem(err[2])
            super(DebugTestResult, self).addError(test, err)

        def addFailure(self, test, err):
            traceback.print_exception(*err)
            if enable_debug:
                pdb.post_mortem(err[2])
            super(DebugTestResult, self).addFailure(test, err)

    return unittest.TextTestRunner(
        verbosity=verbosity,
        failfast=failfast,
        resultclass=DebugTestResult,
    )


def print_suite(suite):
    if hasattr(suite, '__iter__'):
        for x in suite:
            print_suite(x)
    elif hasattr(suite, "_testMethodName"):
        name = getattr(suite, "_testMethodName")
        print(f"{name}")
    else:
        print("invalid")


def filter_suite(suite, func, compared_to: str):
    for testmodule in suite:
        for testsuite in testmodule:
            tests_to_remove = []
            for index, testcase in enumerate(testsuite._tests):
                if func(testcase._testMethodName, compared_to):
                    tests_to_remove.append(index)

            # do this in reverse order, so index does not change
            for index in reversed(tests_to_remove):
                testsuite._tests.pop(index)
    return suite


def filter_before(suite, before: str):
    def is_before(testname: str, compared_to: str) -> bool:
        return testname > compared_to
    return filter_suite(suite, is_before, before)


def filter_after(suite, after: str):
    def is_after(testname: str, compared_to: str) -> bool:
        return testname < compared_to
    return filter_suite(suite, is_after, after)


def filter_exclude(suite, exclude: str):
    def is_excluded(testname: str, compared_to: str) -> bool:
        return exclude in testname
    return filter_suite(suite, is_excluded, exclude)


def live_test(*args):
    args = parse_args(*args)
    env = os.environ
    if args.url:
        env[CT_URL] = args.url
    if args.api_key:
        env[CT_API_KEY] = args.api_key
    env[CT_TEST_LOG_COMMANDS] = str(int(args.log_commands))
    env[CT_TEST_LOG_OUTPUT] = str(int(args.log_output))
    if args.job_id:
        print(f"JOB_ID: {args.job_id}")
        env[CT_TEST_JOB_ID] = args.job_id

    cli = get_cli_base_cmd()
    subprocess.run(cli + "config current -x", shell=True)

    # propagate the debug flags
    if args.debug:
        args.pdb = True
        args.failfast = True

    test_directory = '.'
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

    runner = debugTestRunner(
        enable_debug=args.pdb, verbosity=args.verbosity, failfast=args.failfast
    )
    test_result = runner.run(suite)
    rval = 0
    if len(test_result.errors):
        rval += 1
    if len(test_result.failures):
        rval += 2
    return rval


if __name__ == "__main__":
    sys.exit(live_test(sys.argv[1:]))

import argparse
import os
import pdb
import sys
import traceback
import unittest

from testcase import CT_API_KEY, CT_URL
from testcase import CT_TEST_JOB_ID, CT_TEST_LOG_COMMANDS, CT_TEST_LOG_OUTPUT
from testcase import DEFAULT_SERVER_URL


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
        "--debug",
        dest="debug",
        action='store_true',
        help="Open the debugger when a test fails"
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
    # TODO: Rick Porter 5/21 - add test case filtering
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
        env[CT_TEST_JOB_ID] = args.job_id

    test_directory = '.'
    suite = unittest.TestLoader().discover(test_directory, pattern=args.file_filter)

    runner = debugTestRunner(
        enable_debug=args.debug, verbosity=args.verbosity, failfast=args.failfast
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

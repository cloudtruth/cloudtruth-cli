import argparse
import os
import pdb
import sys
import traceback
import unittest

from testcase import CT_API_KEY, CT_URL, DEFAULT_SERVER_URL


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
    # TODO: add test case filtering
    return parser.parse_args(*args)


def debugTestRunner(enable_debug: bool, verbosity: int, failfast: bool):
    """Overload the TextTestRunner to conditionally drop into pdb on an error/failure."""
    class DebugTestResult(unittest.TextTestResult):
        def addError(self, test, err):
            # called before tearDown()
            if enable_debug:
                traceback.print_exception(*err)
                pdb.post_mortem(err[2])
            super(DebugTestResult, self).addError(test, err)

        def addFailure(self, test, err):
            if enable_debug:
                traceback.print_exception(*err)
                pdb.post_mortem(err[2])
            super(DebugTestResult, self).addFailure(test, err)

    return unittest.TextTestRunner(
        verbosity=verbosity,
        failfast=failfast,
        resultclass=DebugTestResult,
    )


def live_test(*args):
    args = parse_args(*args)
    if args.url is None:
        args.url = os.environ(CT_API_KEY)

    env = os.environ.copy()
    if args.url:
        env[CT_URL] = args.url
    if args.api_key:
        env[CT_API_KEY] = args.api_key

    test_directory = '.'
    suite = unittest.TestLoader().discover(test_directory, pattern=args.file_filter)

    runner = debugTestRunner(
        enable_debug=args.debug, verbosity=args.verbosity, failfast=args.failfast
    )
    runner.run(suite)


if __name__ == "__main__":
    sys.exit(live_test(sys.argv[1:]))

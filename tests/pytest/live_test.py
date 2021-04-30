import argparse
import os
import sys
import unittest

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
        "-v",
        "--verbosity",
        type=int,
        default=1,
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
        help="Filter the files run using the specified pattern"
    )
    # TODO: add test file filtering, add test case filtering
    return parser.parse_args(*args)


def live_test(*args):
    result = 0
    args = parse_args(*args)
    if args.url is None:
        args.url = os.environ("CLOUDTRUTH_API_KEY")

    env = os.environ.copy()
    if args.url:
        env["CLOUDTRUTH_SERVER_URL"] = args.url
    if args.api_key:
        env["CLOUDTRUTH_API_KEY"] = args.api_key

    # TODO: file-filter
    test_directory = '.'
    suite = unittest.TestLoader().discover(test_directory)

    if args.debug:
        suite.debug()
    else:
        unittest.TextTestRunner(verbosity=args.verbosity).run(suite)


if __name__ == "__main__":
    sys.exit(live_test(sys.argv[1:]))

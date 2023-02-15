import argparse
import dataclasses
import subprocess
import sys
from datetime import datetime
from typing import List

from testcase import get_cli_base_cmd
from testcase import Result

base_cmd = get_cli_base_cmd()


@dataclasses.dataclass
class CleanupItem:
    name: str
    list_cmd: str
    del_cmd: str
    items: List = dataclasses.field(default_factory=list)


def parse_args(*args) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Cleanup the CloudTruth environment")
    parser.add_argument(
        dest="needles",
        nargs="*",
        default=["Windows", "Linux", "macOS", "ci-cli", "testcli"],
        help="Search strings to look for",
    )
    parser.add_argument(
        "-q",
        "--quiet",
        dest="quiet",
        action="store_true",
        help="Do not show what the script is doing",
    )
    parser.add_argument("-v", "--verbose", dest="verbose", action="store_true", help="Detailed output")
    parser.add_argument("--confirm", "--yes", dest="confirm", action="store_true", help="Skip confirmation prompt")
    return parser.parse_args(*args)


def cli(cmd: str) -> Result:
    updated = base_cmd + cmd.replace("'", '"')  # allows this to work on Windows
    start = datetime.now()
    process = subprocess.run(updated, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    delta = datetime.now() - start
    return Result(
        return_value=process.returncode,
        stdout=process.stdout.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
        stderr=process.stderr.decode("us-ascii", errors="ignore").replace("\r", "").split("\n"),
        timediff=delta,
        command=updated,
    )


def yes_or_no(question: str) -> bool:
    reply = str(input(question + " (y/n): ")).lower().strip()
    if reply[0] == "y":
        return True
    if reply[0] == "n":
        return False
    else:
        return yes_or_no("Please enter ")


def cloudtruth_cleanup(*args):
    args = parse_args(*args)
    if not args.needles:
        print("No search strings provided")
        return -1
    # reset verbosity flags if they conflict with each other
    if args.quiet and args.verbose:
        args.quiet = False
        args.verbose = False

    result = cli("config curr -x")
    print(result.command)
    print(result.out())

    # define a set of elements to cleanup
    elements = [
        CleanupItem(name="projects", list_cmd="proj tree", del_cmd="proj del -y"),
        CleanupItem(name="environments", list_cmd="env tree", del_cmd="env del -y"),
        CleanupItem(name="users", list_cmd="user ls", del_cmd="user del -y"),
        CleanupItem(name="groups", list_cmd="group ls", del_cmd="group del -y"),
        CleanupItem(name="invitations", list_cmd="user invite ls", del_cmd="user invite del -y"),
        CleanupItem(name="types", list_cmd="types tree", del_cmd="types del -y"),
        CleanupItem(name="pushes", list_cmd="action push ls", del_cmd="action push del -y"),
        CleanupItem(name="imports", list_cmd="action import ls", del_cmd="action import del -y"),
    ]

    for elem in elements:
        if not args.quiet:
            print(f"Looking for matching {elem.name}...")
        result = cli(elem.list_cmd)

        # use reverse order to accommodate the ordering `tree` commands
        for line in reversed(result.stdout):
            item = line.strip()
            if any([needle for needle in args.needles if needle in item]):
                elem.items.append(line.strip())

    if not (any([x for x in elements if x.items])):
        types = [x.name for x in elements]
        type_list = ", ".join(types[:-1])
        type_list += f", or {types[-1]}"
        search_list = ", ".join(args.needles)
        print(f"No {type_list} items found matching: {search_list}")
        return 0

    if not (args.confirm and args.quiet):
        print("\n\nFound matches: ")
        for elem in elements:
            if not elem.items:
                print(f"  {elem.name}: None")
            else:
                print(f"  {elem.name}:")
                for item in elem.items:
                    print(f"    {item}")
        print("")

    if not args.confirm and not yes_or_no("Delete the above items"):
        print("No items deleted")
        return 0

    all_deleted = True
    for elem in elements:
        if not elem.items:
            continue
        for item in elem.items:
            result = cli(elem.del_cmd + f" '{item}'")
            if args.verbose:
                print(result.command)
            if result.return_value != 0:
                all_deleted = False
                print(f"Failed to delete {elem.name} {item}")
                if args.verbose:
                    print(result.err())

    if all_deleted:
        print("Deleted all items")
    return 0


if __name__ == "__main__":
    sys.exit(cloudtruth_cleanup(sys.argv[1:]))

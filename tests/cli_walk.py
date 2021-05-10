import argparse
import difflib
import re
import subprocess
import sys
from pathlib import Path
from typing import List


SUBCOMMANDS = "SUBCOMMANDS:"  # NOTE: the ':' is important to denote the start of the section
SUBCOMMAND_RE = re.compile(r"^\W+(?P<subcommand>\S+)\W+")
SEPARATOR = "===================="
MAX_SEP_LEVELS = 3


def parse_args(*args) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Walk a CLI with all subcommands, and print the help to the specified file."
    )
    parser.add_argument(
        dest="purpose",
        type=str,
        choices=["check", "generate"],
        help="Action to perform",
    )
    parser.add_argument(
        "-x",
        "--executable",
        dest="executable",
        type=str,
        help="Cli executable to run",
    )
    parser.add_argument(
        "-f",
        "--file",
        dest="file",
        type=str,
        default="help.txt",
        help="File for checking against, or reading from"
    )
    return parser.parse_args(*args)


def get_cli_base_cmd() -> str:
    """
    Returns the path to the executable (presumably).
    """
    # walk back up looking for top of projects, and goto `target/debug/cloudtruth`
    curr = Path(__file__).absolute()
    subdir = Path("target") / "debug"
    match = False
    while not match and curr:
        possible = curr.parent / subdir
        match = possible.exists()
        curr = curr.parent

    if not match:
        return "cloudtruth"

    for filename in ("cloudtruth", "cloudtruth.exe"):
        file = possible / filename
        if file.exists():
            return str(file)

    # this is a little odd... no executable found in "local" directories
    return "cloudtruth"


def find_subcommands(help_str: str) -> List[str]:
    subcommands = []

    subsection = False
    lines = help_str.split("\n")
    for line in lines:
        if not subsection:
            subsection = SUBCOMMANDS in line
            continue

        match  = SUBCOMMAND_RE.search(line)
        if match:
            subcommands.append(match.group("subcommand"))

    return subcommands


def walk_output(cmd: str, sep_levels: int) -> str:
    help_cmd = f"{cmd} -h"
    sub_proc = subprocess.run(help_cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    help_str = sub_proc.stdout.decode("us-ascii", errors="ignore").replace("\r", "")
    result = help_str

    # recursively go through all the subcommands
    subcommands = find_subcommands(help_str)
    for sub_cmd in subcommands:
        sub_help = walk_output(cmd + " " + sub_cmd, sep_levels - 1)
        if sub_help:
            result += sep_levels * SEPARATOR + "\n" + sub_help

    return result


def walk_cli(*args):
    result = -1
    args = parse_args(*args)

    cmd = args.executable or get_cli_base_cmd()
    current = walk_output(cmd, MAX_SEP_LEVELS)
    if args.purpose == "check":
        with open(args.file) as fp:
            last = fp.read().splitlines()[:-1]  # remove line-endings and extra newline
            current = current.splitlines()
            if current != last:
                diff = difflib.unified_diff(last, current, "Previous", "Current")
                print("Detected CLI changes:")
                print("\n".join(diff))
                print("You can run 'python3 cli_walk.py generate' if the changes are intentional.")
            else:
                result = 0
    elif args.purpose == "generate":
        with open(args.file, mode="w") as fp:
            print(current, file=fp)
            result = 0
    return result


if __name__ == "__main__":
    sys.exit(walk_cli(sys.argv[1:]))

# -*- coding: utf-8 -*-
import sys
import argparse

import yaml

from pathlib import Path
from jinja2 import Template

TEMPLATE_DIR = "templates"
DOCKER_DIR = "docker"
CONFIG_FILE = "cfg.yaml"


def parse_args(*args) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Update the workflow and dockerfiles for CI"
    )
    parser.add_argument(
        "-c",
        "--config-file",
        dest="config_file",
        type=str,
        help="Input YAML file with configuration for generating workflow and docker files.",
        default=CONFIG_FILE,
    )
    parser.add_argument(
        "-d",
        "--docker-dir",
        dest="docker_dir",
        type=str,
        help="Directory to output supporting docker files.",
        default=DOCKER_DIR,
    )
    parser.add_argument(
        "-t",
        "--template-dir",
        dest="template_dir",
        type=str,
        help="Directory containing input workflow YAML templates.",
        default=TEMPLATE_DIR,
    )
    parser.add_argument(
        "-w",
        "--workflow",
        dest="workflow_name",
        type=str,
        choices=["draft", "prerelease"],
    )
    return parser.parse_args(*args)


def update_workflow(config_file: str, template_dir: str, workflow_name: str) -> None:
    with Path(f"{config_file}").open() as fp:
        config = yaml.safe_load(fp.read())

    with Path(f"{template_dir}/{workflow_name}/workflow-job.tmpl").open() as fp:
        job = fp.read()

    with Path(f"{template_dir}/{workflow_name}/workflow-step-direct-ps.tmpl").open() as fp:
        step_direct_ps = fp.read()

    with Path(f"{template_dir}/{workflow_name}/workflow-step-direct-sh.tmpl").open() as fp:
        step_direct_sh = fp.read()

    with Path(f"{template_dir}/{workflow_name}/workflow-step-docker.tmpl").open() as fp:
        step_docker = fp.read()

    with Path(f"{template_dir}/{workflow_name}/workflow-header.yaml").open() as fp:
        new_workflow = fp.read()

    for os, data in config["jobs"].items():
        jt = Template(job)
        if data["docker"]:
            # multiple steps per job
            new_workflow = new_workflow + jt.render(os=os, runs_on="ubuntu-latest")
            for version in data["versions"]:
                st = Template(step_docker)
                new_workflow += st.render(os=os, version=version)
        else:
            for version in data["versions"]:
                runs_on = f"{os}-{version}"
                new_workflow = new_workflow + jt.render(os=os, runs_on=runs_on)
                st = Template(step_direct_ps if data.get("powershell") else step_direct_sh)
                new_workflow += st.render(os=os, version=version)

    if workflow_name == "draft":
        # we have to merge it in
        with Path("../.github/workflows/create-draft-release.yml").open() as fp:
            existing_workflow_lines = fp.readlines()
        for line in range(len(existing_workflow_lines)):
            if "## @@@" in existing_workflow_lines[line]:
                existing_workflow = "".join(existing_workflow_lines[:(line + 1)])
                break
        assert existing_workflow, "marker not found in create-draft-release?"
        existing_workflow += new_workflow
        with Path("../.github/workflows/create-draft-release.yml").open("w") as fp:
            fp.write(existing_workflow)

    elif workflow_name == "prerelease":
        # we can replace it
        with Path("../.github/workflows/check-pre-release.yml").open("w") as fp:
            fp.write(new_workflow)


def update_dockerfiles(config_file: str, template_dir: str, workflow_name: str, docker_dir: str) -> None:
    with Path(f"{config_file}").open() as fp:
        config = yaml.safe_load(fp.read())

    with Path(f"{template_dir}/{workflow_name}/Dockerfile.tmpl").open() as fp:
        dockerfile = fp.read()

    files = []
    for os, data in config["jobs"].items():
        if not data["docker"]:
            continue

        for version in data["versions"]:
            dt = Template(dockerfile)
            filename = f"{docker_dir}/{workflow_name}/Dockerfile.{os}-{version}"
            with Path(filename).open("w") as fp:
                fp.write(dt.render(os=os, version=version))
                files += [filename]

    print(f"Updated files in {docker_dir}")


def main(*sys_args):
    args = parse_args(*sys_args)
    update_dockerfiles(args.config_file, args.template_dir, args.workflow_name, args.docker_dir)
    update_workflow(args.config_file, args.template_dir, args.workflow_name)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

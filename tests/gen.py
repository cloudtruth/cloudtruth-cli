# -*- coding: utf-8 -*-
import sys
import argparse

import yaml

from pathlib import Path
from jinja2 import Template


TEMPLATE_DIR = "templates"
DOCKER_DIR = "docker"
WORKFLOW_FILE = "test.yaml"
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
        help="Directory for output docker files.",
        default=DOCKER_DIR,
    )
    parser.add_argument(
        "-t",
        "--template-dir",
        dest="template_dir",
        type=str,
        help="File for output workflow YAML data.",
        default=TEMPLATE_DIR,
    )
    parser.add_argument(
        "-w",
        "--workflow-file",
        dest="workflow_file",
        type=str,
        help="File for output workflow YAML data.",
        default=WORKFLOW_FILE,
    )
    parser.add_argument(
        "--skip-workflow",
        dest="skip_workflow",
        action="store_true",
        help="Skip the workflow file updates."
    )
    parser.add_argument(
        "--skip-docker",
        dest="skip_docker",
        action="store_true",
        help="Skip the docker file updates."
    )
    return parser.parse_args(*args)


def update_workflow(config_file: str, template_dir: str, workflow_file: str) -> None:
    with Path(f"{config_file}").open() as fp:
        config = yaml.safe_load(fp.read())

    with Path(f"{template_dir}/workflow-job.tmpl").open() as fp:
        job = fp.read()

    with Path(f"{template_dir}/workflow-step-direct-ps.tmpl").open() as fp:
        step_direct_ps = fp.read()

    with Path(f"{template_dir}/workflow-step-direct-sh.tmpl").open() as fp:
        step_direct_sh = fp.read()

    with Path(f"{template_dir}/workflow-step-docker.tmpl").open() as fp:
        step_docker = fp.read()

    with Path(f"{template_dir}/workflow-header.yaml").open() as fp:
        workflow = fp.read()

    for os, data in config["jobs"].items():
        jt = Template(job)
        if data["docker"]:
            # multiple steps per job
            workflow = workflow + jt.render(os=os, runs_on="ubuntu-latest")
            for version in data["versions"]:
                st = Template(step_docker)
                workflow += st.render(os=os, version=version)
        else:
            for version in data["versions"]:
                runs_on = f"{os}-{version}"
                workflow = workflow + jt.render(os=os, runs_on=runs_on)
                st = Template(step_direct_ps if data.get("powershell") else step_direct_sh)
                workflow += st.render(os=os, version=version)

    with Path(f"{workflow_file}").open("w") as fp:
        fp.write(workflow)
    print(f"Data from {workflow_file} should be merged into .github/workflows/ files")


def update_dockerfiles(config_file: str, template_dir: str, docker_dir: str) -> None:
    with Path(f"{config_file}").open() as fp:
        config = yaml.safe_load(fp.read())

    with Path(f"{template_dir}/Dockerfile.tmpl").open() as fp:
        dockerfile = fp.read()

    files = []
    for os, data in config["jobs"].items():
        if not data["docker"]:
            continue

        for version in data["versions"]:
            dt = Template(dockerfile)
            filename = f"{docker_dir}/Dockerfile.{os}-{version}"
            with Path(filename).open("w") as fp:
                fp.write(dt.render(os=os, version=version))
                files += [filename]

    print(f"Updated files in {docker_dir}")


def main(*sys_args):
    args = parse_args(*sys_args)
    if not args.skip_docker:
        update_dockerfiles(args.config_file, args.template_dir, args.docker_dir)
    if not args.skip_workflow:
        update_workflow(args.config_file, args.template_dir, args.workflow_file)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

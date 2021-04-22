# -*- coding: utf-8 -*-
from pathlib import Path

import yaml

from jinja2 import Template


template_dir="templates"
docker_dir="docker"
output_file="test.yaml"
config_file="cfg.yaml"


with Path(f"{config_file}").open() as fp:
    config = yaml.safe_load(fp.read())

with Path(f"{template_dir}/Dockerfile.tmpl").open() as fp:
    dockerfile = fp.read()

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

            dt = Template(dockerfile)
            with Path(f"{docker_dir}/Dockerfile.{os}-{version}").open("w") as fp:
                fp.write(dt.render(os=os, version=version))
    else:
        for version in data["versions"]:
            runs_on = f"{os}-{version}"
            workflow = workflow + jt.render(os=os, runs_on=runs_on)
            st = Template(step_direct_ps if data.get("powershell") else step_direct_sh)
            workflow += st.render(os=os, version=version)

with Path(f"{output_file}").open("w") as fp:
    fp.write(workflow)

print(f"Data from ${output_file} should be merged into .github/workflows/ files")

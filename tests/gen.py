# -*- coding: utf-8 -*-
from pathlib import Path

import yaml

from jinja2 import Template


with Path("cfg.yaml").open() as fp:
    config = yaml.safe_load(fp.read())

with Path("template/Dockerfile.tmpl").open() as fp:
    dockerfile = fp.read()

with Path("template/workflow-job.tmpl").open() as fp:
    job = fp.read()

with Path("template/workflow-step-direct-ps.tmpl").open() as fp:
    step_direct_ps = fp.read()

with Path("template/workflow-step-direct-sh.tmpl").open() as fp:
    step_direct_sh = fp.read()

with Path("template/workflow-step-docker.tmpl").open() as fp:
    step_docker = fp.read()

with Path("template/workflow-header.yaml").open() as fp:
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
            with Path(f"tests/Dockerfile.{os}-{version}").open("w") as fp:
                fp.write(dt.render(os=os, version=version))
    else:
        for version in data["versions"]:
            runs_on = f"{os}-{version}"
            workflow = workflow + jt.render(os=os, runs_on=runs_on)
            st = Template(step_direct_ps if data.get("powershell") else step_direct_sh)
            workflow += st.render(os=os, version=version)

with Path(".github/workflows/test.yaml").open("w") as fp:
    fp.write(workflow)

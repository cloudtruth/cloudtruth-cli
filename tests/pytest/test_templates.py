import unittest

from testcase import TestCase
from testcase import CT_ENV
from testcase import PROP_MODIFIED
from testcase import PROP_NAME
from testcase import PROP_RAW
from testcase import PROP_VALUE
from testcase import REDACTED

# This definition allows us to fake-out flake8, and not have it complain about trailing whitespace.
fake_flake8 = ""


def empty_template(project: str) -> str:
    return f"No templates in project '{project}'"


class TestTemplates(TestCase):
    def test_template_basic(self) -> None:
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-template-proj")
        temp_name = "orig-template"  # templates are scoped to a project, so no need to "randomize"
        filename = self.make_name("basic-body") + ".txt"
        body = "Text with no params\n"
        self.write_file(filename, body)
        empty_msg = empty_template(proj_name)

        self.create_project(cmd_env, proj_name)

        sub_cmd = base_cmd + f"--project '{proj_name}' template "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertIn(empty_msg, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --desc \"{orig_desc}\" --body '{filename}'")
        self.assertResultSuccess(result)
        self.assertIn(f"Created template '{temp_name}'", result.out())

        result = self.run_cli(cmd_env, sub_cmd + "ls -v -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{temp_name},{orig_desc}", result.out())

        # check that we get back the "evaluated" text
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertResultSuccess(result)
        self.assertEqual(body, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"validate {temp_name}")
        self.assertResultSuccess(result)
        self.assertIn("Success", result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated template '{temp_name}'", result.out())
        result = self.run_cli(cmd_env, sub_cmd + "ls --values -f csv")
        self.assertResultSuccess(result)
        self.assertIn(f"{temp_name},{new_desc}", result.out())

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --desc \"{new_desc}\"")
        self.assertResultSuccess(result)

        # rename
        orig_name = temp_name
        temp_name = "renamed-temp"
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{temp_name}\"")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated template '{temp_name}'", result.out())
        result = self.run_cli(cmd_env, sub_cmd + "ls")
        self.assertResultSuccess(result)
        self.assertIn(temp_name, result.out())
        self.assertNotIn(orig_name, result.out())

        # attempting to get template that does not exist yield error
        result = self.run_cli(cmd_env, sub_cmd + f"get '{orig_name}'")
        self.assertResultError(result, f"No template '{orig_name}' found in project '{proj_name}'")

        # change the body
        body = "different fixed value\n"
        self.write_file(filename, body)
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --body \"{filename}\"")
        self.assertResultSuccess(result)
        self.assertIn(f"Updated template '{temp_name}'", result.out())

        # check the new body text
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertResultSuccess(result)
        self.assertEqual(body, result.out())

        # nothing variable, but quick test of API
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} --raw")
        self.assertResultSuccess(result)
        self.assertEqual(body, result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name}")
        self.assertResultWarning(
            result,
            f"Template '{temp_name}' not updated: no updated parameters provided",
        )

        # test the list without the values
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertResultSuccess(result)
        self.assertIn(temp_name, result.out())
        self.assertNotIn(new_desc, result.out())

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {temp_name} --confirm")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertResultSuccess(result)
        self.assertNotIn(temp_name, result.out())

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {temp_name} --confirm")
        self.assertResultWarning(result, f"Template '{temp_name}' does not exist for project '{proj_name}'")

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)

    def test_template_evaluate_environments(self) -> None:
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("test-temp-eval")
        temp_name = "my_template"  # templates are scoped to a project, so no need to "randomize"
        env_a = self.make_name("env_eval_a")
        env_b = self.make_name("env_eval_b")
        param1 = "param1"
        param2 = "secret1"
        value1a = "some val with space"
        value1b = "diff_env_value"
        value2a = "sssshhhhhhh"
        value2b = "top-secret"

        # create infrastructure
        self.create_project(cmd_env, proj_name)
        self.create_environment(cmd_env, env_a)
        self.create_environment(cmd_env, env_b)
        self.set_param(cmd_env, proj_name, param1, value1a, env=env_a)
        self.set_param(cmd_env, proj_name, param1, value1b, env=env_b)
        self.set_param(cmd_env, proj_name, param2, value2a, env=env_a, secret=True)
        self.set_param(cmd_env, proj_name, param2, value2b, env=env_b, secret=True)

        filename = self.make_name("eval-body") + ".txt"
        base = """\
# here is a comment
// we do not care about what other content you put in
simple.param=PARAM1
ANOTHER_PARAM=PARAM2
"""
        body = base.replace("PARAM1", f"{{{{{param1}}}}}").replace("PARAM2", f"{{{{{param2}}}}}")
        eval_a = base.replace("PARAM1", value1a).replace("PARAM2", REDACTED)
        eval_b = base.replace("PARAM1", value1b).replace("PARAM2", REDACTED)
        self.write_file(filename, body)

        sub_cmd = base_cmd + f"--project {proj_name} template "
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} -b {filename}")
        self.assertResultSuccess(result)

        ##########################
        # Check environment A
        cmd_env[CT_ENV] = env_a

        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertResultSuccess(result)
        self.assertIn(eval_a, result.out())

        # see that we get back the unresolved/unevaluated body
        result = self.run_cli(cmd_env, sub_cmd + f"get -r {temp_name}")
        self.assertResultSuccess(result)
        self.assertIn(body, result.out())

        # check preview, too
        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename}")
        self.assertResultSuccess(result)
        self.assertIn(eval_a, result.out())

        # now, display the secrets
        eval_a = eval_a.replace(REDACTED, f"{value2a}")
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} -s")
        self.assertResultSuccess(result)
        self.assertIn(eval_a, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename} --secrets")
        self.assertResultSuccess(result)
        self.assertIn(eval_a, result.out())

        ##########################
        # Check environment B
        cmd_env[CT_ENV] = env_b

        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertResultSuccess(result)
        self.assertIn(eval_b, result.out())

        # see that we get back the unresolved/unevaluated body
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} -r")
        self.assertResultSuccess(result)
        self.assertIn(body, result.out())

        # check preview, too
        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename}")
        self.assertResultSuccess(result)
        self.assertIn(eval_b, result.out())

        # now, display the secrets
        eval_b = eval_b.replace(REDACTED, f"{value2b}")
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} -s")
        self.assertResultSuccess(result)
        self.assertIn(eval_b, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename} --secrets")
        self.assertResultSuccess(result)
        self.assertIn(eval_b, result.out())

        ############
        # see that we cannot delete a parameter with the template using it
        result = self.run_cli(cmd_env, base_cmd + f"--project {proj_name} param del -y '{param1}' ")
        self.assertResultError(result, f"Cannot delete {param1} as it is used in the following templates: {temp_name}")

        ###########
        # check error message with unresolved variables
        no_param = "my-missing-param"
        body = base.replace("PARAM1", f"{{{{{no_param}}}}}").replace("PARAM2", f"{{{{{param2}}}}}")
        self.write_file(filename, body)
        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename} --secrets")
        self.assertResultError(result, "Template references parameter(s) that do not exist")
        self.assertIn(no_param, result.err())

        # cleanup
        self.delete_file(filename)
        self.delete_environment(cmd_env, env_b)
        self.delete_environment(cmd_env, env_a)
        self.delete_project(cmd_env, proj_name)

    def test_template_as_of_time(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-temp-times")
        self.create_project(cmd_env, proj_name)

        ##################
        # create a couple parameters
        param1 = "some_param"
        value1a = "value first"
        value1b = "value second"
        self.set_param(cmd_env, proj_name, param1, value1a)

        param2 = "another-param"
        value2a = "devops"
        value2b = "sre"
        self.set_param(cmd_env, proj_name, param2, value2a)

        #################
        # create a template
        temp_name = "my-test-template"
        temp_cmd = base_cmd + f"--project '{proj_name}' template "
        filename = self.make_name("test-temp-times") + ".txt"

        body = """\
# just a different template
references = PARAM
"""
        self.write_file(filename, body.replace("PARAM", f"{{{{{param1}}}}}"))
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp_name}' -b {filename}")
        self.assertResultSuccess(result)

        result = self.run_cli(cmd_env, temp_cmd + "list --show-times -f json")
        self.assertResultSuccess(result)
        item = eval(result.out()).get("template")[0]
        modified_at = item.get(PROP_MODIFIED)

        #################
        # update values
        self.set_param(cmd_env, proj_name, param1, value1b)
        self.set_param(cmd_env, proj_name, param2, value2b)

        self.write_file(filename, body.replace("PARAM", f"{{{{{param2}}}}}"))
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp_name}' -b {filename}")
        self.assertResultSuccess(result)

        ##################
        # check template get
        get_cmd = temp_cmd + f"get '{temp_name}' "
        result = self.run_cli(cmd_env, get_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{value2b}"))

        result = self.run_cli(cmd_env, get_cmd + "--raw")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{{{{{param2}}}}}"))

        result = self.run_cli(cmd_env, get_cmd + f"--as-of '{modified_at}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{value1a}"))

        result = self.run_cli(cmd_env, get_cmd + f"--raw --as-of '{modified_at}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{{{{{param1}}}}}"))

        # this is before the project exists
        result = self.run_cli(cmd_env, get_cmd + "--as-of 2020-02-02")
        self.assertResultError(result, "No HistoricalProject matches the given query")

        ##################
        # check preview
        body = """\
# just a comment
this.is.a.template.value=PARAM1
"""
        self.write_file(filename, body.replace("PARAM1", f"{{{{{param1}}}}}"))

        # check the current evaluation
        preview_cmd = temp_cmd + f"preview '{filename}' "
        result = self.run_cli(cmd_env, preview_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM1", value1b + "\n"))

        # check the earlier evaluation
        result = self.run_cli(cmd_env, preview_cmd + f"--as-of '{modified_at}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM1", value1a + "\n"))

        # this is before the project exists
        result = self.run_cli(cmd_env, preview_cmd + "--as-of 2020-02-02")
        self.assertResultError(result, "No HistoricalProject matches the given query")

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)

    def test_template_as_of_tag(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-temp-tag")
        self.create_project(cmd_env, proj_name)

        # add an environment (needed for tagging)
        env_name = self.make_name("test-tag-temp")
        self.create_environment(cmd_env, env_name)

        ##################
        # create a couple parameters
        param1 = "some_param"
        value1a = "value first"
        value1b = "value second"
        self.set_param(cmd_env, proj_name, param1, value1a, env=env_name)

        param2 = "another-param"
        value2a = "devops"
        value2b = "sre"
        self.set_param(cmd_env, proj_name, param2, value2a, env=env_name)

        #################
        # create a template
        temp_name = "my-test-template"
        temp_cmd = base_cmd + f"--project '{proj_name}' --env '{env_name}' template "
        filename = self.make_name("test-temp-tag") + ".txt"

        body = """\
# just a different template
references = PARAM
"""
        self.write_file(filename, body.replace("PARAM", f"{{{{{param1}}}}}"))
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp_name}' -b {filename}")
        self.assertResultSuccess(result)

        # put down a tag
        tag_name = "template-tag"
        result = self.run_cli(cmd_env, base_cmd + f"env tag set '{env_name}' '{tag_name}'")
        self.assertResultSuccess(result)

        #################
        # update values
        self.set_param(cmd_env, proj_name, param1, value1b, env=env_name)
        self.set_param(cmd_env, proj_name, param2, value2b, env=env_name)

        self.write_file(filename, body.replace("PARAM", f"{{{{{param2}}}}}"))
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp_name}' -b {filename}")
        self.assertResultSuccess(result)

        ##################
        # check template get
        get_cmd = temp_cmd + f"get '{temp_name}' "
        result = self.run_cli(cmd_env, get_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{value2b}"))

        result = self.run_cli(cmd_env, get_cmd + "--raw")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{{{{{param2}}}}}"))

        result = self.run_cli(cmd_env, get_cmd + f"--as-of '{tag_name}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{value1a}"))

        result = self.run_cli(cmd_env, get_cmd + f"--raw --as-of '{tag_name}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM", f"{{{{{param1}}}}}"))

        # this is before the project exists
        missing_tag = "my-missing-tag"
        err_msg = f"Tag `{missing_tag}` could not be found in environment `{env_name}`"
        result = self.run_cli(cmd_env, get_cmd + f"--as-of {missing_tag}")
        self.assertResultError(result, err_msg)

        ##################
        # check preview
        body = """\
# just a comment
this.is.a.template.value=PARAM1
"""
        self.write_file(filename, body.replace("PARAM1", f"{{{{{param1}}}}}"))

        # check the current evaluation
        preview_cmd = temp_cmd + f"preview '{filename}' "
        result = self.run_cli(cmd_env, preview_cmd)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM1", value1b + "\n"))

        # check the earlier evaluation
        result = self.run_cli(cmd_env, preview_cmd + f"--as-of '{tag_name}'")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), body.replace("PARAM1", value1a + "\n"))

        # this is before the project exists
        missing_tag = "another-tag-gone"
        err_msg = f"Tag `{missing_tag}` could not be found in environment `{env_name}`"
        result = self.run_cli(cmd_env, preview_cmd + f"--as-of {missing_tag}")
        self.assertResultError(result, err_msg)

        # cleanup
        self.delete_file(filename)
        self.delete_environment(cmd_env, env_name)
        self.delete_project(cmd_env, proj_name)

    @unittest.skip("Waiting on server fix")
    def test_template_history(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-temp-hist")
        self.create_project(cmd_env, proj_name)

        env_name = self.make_name("env-temp-hist")
        self.create_environment(cmd_env, env_name)

        temp_cmd = base_cmd + f"--project {proj_name} --env {env_name} temp "

        # take a baseline before we have any template history
        result = self.run_cli(cmd_env, temp_cmd + "history")
        self.assertResultSuccess(result)
        self.assertIn("No template history in project", result.out())

        temp1 = "temp1"
        body1a = "first body"
        body1b = "second body"
        desc1 = "simple desc"
        temp2 = "temp2"
        body2a = "# bogus text"
        body2b = "different temp text"
        filename = "history-template.txt"

        # create the templates
        self.write_file(filename, body1a)
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp1}' -b '{filename}' -d '{desc1}'")
        self.assertResultSuccess(result)

        self.write_file(filename, body2a)
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp2}' -b '{filename}'")
        self.assertResultSuccess(result)

        # get the modified time -- before making changes
        result = self.run_cli(cmd_env, temp_cmd + "list --show-times -f json")
        self.assertResultSuccess(result)
        temp_info = eval(result.out())
        modified_at = temp_info.get("template")[1].get("Modified At")

        tag_name = "stable"
        result = self.run_cli(cmd_env, base_cmd + f"env tag set {env_name} {tag_name}")
        self.assertResultSuccess(result)

        # update the template bodies
        self.write_file(filename, body1b)
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp1}' -b '{filename}'")
        self.assertResultSuccess(result)

        self.write_file(filename, body2b)
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp2}' -b '{filename}'")
        self.assertResultSuccess(result)

        user = self.current_username(cmd_env)

        # get a complete history
        result = self.run_cli(cmd_env, temp_cmd + "history -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Date,User,Action,Name,Changes", result.out())
        self.assertIn(f",{user},create,{temp1},", result.out())
        self.assertIn(body1a, result.out())
        self.assertIn(f",{user},update,{temp1},", result.out())
        self.assertIn(body1b, result.out())
        self.assertIn(desc1, result.out())
        self.assertIn(f",{user},create,{temp2},", result.out())
        self.assertIn(body2a, result.out())
        self.assertIn(f",{user},update,{temp2},", result.out())
        self.assertIn(body2b, result.out())

        # get a focused history on just one
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp2}' -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn("Date,User,Action,Name,Changes", result.out())
        self.assertNotIn(temp1, result.out())
        self.assertNotIn(body1a, result.out())
        self.assertNotIn(body1b, result.out())
        self.assertNotIn(desc1, result.out())
        self.assertIn("Date,User,Action,Changes", result.out())  # drop Name since it is given
        self.assertIn(temp2, result.out())
        self.assertIn(body2a, result.out())
        self.assertIn(body2b, result.out())

        # further focus on older updates using a time
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp2}' --as-of '{modified_at}'")
        self.assertResultSuccess(result)
        self.assertIn(temp2, result.out())
        self.assertIn(body2a, result.out())
        self.assertNotIn(body2b, result.out())  # filtered out by time

        # further focus on older updates using tag
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp2}' --as-of '{tag_name}'")
        self.assertResultSuccess(result)
        self.assertIn(temp2, result.out())
        self.assertIn(body2a, result.out())
        self.assertNotIn(body2b, result.out())  # filtered out by time

        # delete both
        result = self.run_cli(cmd_env, temp_cmd + f"del -y '{temp2}'")
        self.assertResultSuccess(result)
        result = self.run_cli(cmd_env, temp_cmd + f"del -y '{temp1}'")
        self.assertResultSuccess(result)

        # see that the deleted show up in the full history
        result = self.run_cli(cmd_env, temp_cmd + "history -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Date,User,Action,Name,Changes", result.out())
        self.assertIn(f",{user},delete,{temp1},", result.out())
        self.assertIn(f",{user},delete,{temp2},", result.out())

        # now that it is deleted, see that we fail to resolve the template name
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp1}'")
        self.assertResultError(result, f"No template '{temp1}' found in project '{proj_name}'")

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env_name)

    def test_template_diff(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-template-diff")
        self.create_project(cmd_env, proj_name)

        # add a couple environments
        env_a = self.make_name("ttag-diff-env-a")
        self.create_environment(cmd_env, env_a)
        env_b = self.make_name("ttag-diff-env-b")
        self.create_environment(cmd_env, env_b)

        # add a couple parameters
        param1 = "param1"
        param2 = "secret1"

        # add some parameters to ENV A
        value1a = "some_value"
        value2a = "ssshhhh"
        self.set_param(cmd_env, proj_name, param1, value1a, env=env_a)
        self.set_param(cmd_env, proj_name, param2, value2a, env=env_a, secret=True)

        proj_cmd = base_cmd + f"--project '{proj_name}' "
        sub_cmd = proj_cmd + "temp "

        # setup the template that references both parameters
        body = """\
# This us a comment common to all environments/times
SECRET=PARAM2

# this is a longer comment to
# demonstrated that text
# gets clipped in
# a unified diff (by default)
# it is not important what is here
# just that the unified diff
# does not show
# every line
# even when there
# are
# too
# many
# lines
PARAMETER=PARAM1

"""
        temp_name = "my-template"
        filename = self.make_name("temp-diff") + ".txt"
        self.write_file(filename, body.replace("PARAM1", f"{{{{{param1}}}}}").replace("PARAM2", f"{{{{{param2}}}}}"))
        result = self.run_cli(cmd_env, sub_cmd + f"set '{temp_name}'  -b '{filename}'")
        self.assertResultSuccess(result)

        # first set of comparisons
        diff_cmd = sub_cmd + f"diff '{temp_name}' "
        result = self.run_cli(cmd_env, diff_cmd + f"-e '{env_a}' --env '{env_b}' ")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({env_a} at current)
+++ {temp_name} ({env_b} at current)
@@ -1,5 +1,5 @@
 # This us a comment common to all environments/times
-SECRET={REDACTED}
+SECRET=
 {fake_flake8}
 # this is a longer comment to
 # demonstrated that text
@@ -14,5 +14,5 @@
 # too
 # many
 # lines
-PARAMETER={value1a}
+PARAMETER=
 {fake_flake8}
""")

        # set some stuff in the current default environment
        def_env = self.get_current_config(cmd_env, "Environment")
        value1d = "different"
        value2d = "be qwiet"
        self.set_param(cmd_env, proj_name, param1, value1d)
        self.set_param(cmd_env, proj_name, param2, value2d)

        # show differences (including secrets) between default and
        result = self.run_cli(cmd_env, diff_cmd + f"-e '{env_a}' -s ")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({env_a} at current)
+++ {temp_name} ({def_env} at current)
@@ -1,5 +1,5 @@
 # This us a comment common to all environments/times
-SECRET={value2a}
+SECRET={value2d}
 {fake_flake8}
 # this is a longer comment to
 # demonstrated that text
@@ -14,5 +14,5 @@
 # too
 # many
 # lines
-PARAMETER={value1a}
+PARAMETER={value1d}
 {fake_flake8}
""")

        # now, set some different values
        same = "matchers"
        value2b = "im hunting wabbits"
        self.set_param(cmd_env, proj_name, param1, same, env=env_a)
        self.set_param(cmd_env, proj_name, param1, same, env=env_b)
        self.set_param(cmd_env, proj_name, param2, value2b, env=env_b)

        # set to have more context than the file
        result = self.run_cli(cmd_env, diff_cmd + f"-e '{env_a}' -e '{env_b}' -s --context 1000")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({env_a} at current)
+++ {temp_name} ({env_b} at current)
@@ -1,18 +1,18 @@
 # This us a comment common to all environments/times
-SECRET={value2a}
+SECRET={value2b}
 {fake_flake8}
 # this is a longer comment to
 # demonstrated that text
 # gets clipped in
 # a unified diff (by default)
 # it is not important what is here
 # just that the unified diff
 # does not show
 # every line
 # even when there
 # are
 # too
 # many
 # lines
 PARAMETER={same}
 {fake_flake8}
""")

        # raw: no differences between environments
        result = self.run_cli(cmd_env, diff_cmd + f"-e '{env_a}' -e '{env_b}' --raw")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), "")

        # raw: no differences between environments, even with lots of context
        result = self.run_cli(cmd_env, diff_cmd + f"-e '{env_a}' -e '{env_b}' -r -c 100")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), "")

        # get the original modified date
        result = self.run_cli(cmd_env, proj_cmd + "temp ls --show-times -f json")
        self.assertResultSuccess(result)
        item = eval(result.out()).get("template")[0]
        modified_at = item.get(PROP_MODIFIED)

        #####################
        # Update the template
        body = f"""\
# This us a comment common to all environments/times
SECRET={{{{{param2}}}}}
PARAMETER={{{{{param1}}}}}

"""
        self.write_file(filename, body)
        result = self.run_cli(cmd_env, proj_cmd + f"temp set '{temp_name}' -b '{filename}'")
        self.assertResultSuccess(result)

        #####################
        # check with the time
        result = self.run_cli(cmd_env, diff_cmd + f"--as-of {modified_at} --raw")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({def_env} at {modified_at})
+++ {temp_name} ({def_env} at current)
@@ -1,18 +1,4 @@
 # This us a comment common to all environments/times
 SECRET={{{{{param2}}}}}
-
-# this is a longer comment to
-# demonstrated that text
-# gets clipped in
-# a unified diff (by default)
-# it is not important what is here
-# just that the unified diff
-# does not show
-# every line
-# even when there
-# are
-# too
-# many
-# lines
 PARAMETER={{{{{param1}}}}}
 {fake_flake8}
""")

        result = self.run_cli(cmd_env, diff_cmd + f"--as-of {modified_at} --env {env_a} -s")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({env_a} at {modified_at})
+++ {temp_name} ({def_env} at current)
@@ -1,18 +1,4 @@
 # This us a comment common to all environments/times
-SECRET={value2a}
-
-# this is a longer comment to
-# demonstrated that text
-# gets clipped in
-# a unified diff (by default)
-# it is not important what is here
-# just that the unified diff
-# does not show
-# every line
-# even when there
-# are
-# too
-# many
-# lines
-PARAMETER={value1a}
+SECRET={value2d}
+PARAMETER={value1d}
 {fake_flake8}
""")

        #####################
        # Tag testing
        tag_name = "my-tag"
        result = self.run_cli(cmd_env, proj_cmd + f"env tag set {env_b} {tag_name}")
        self.assertResultSuccess(result)

        # compare a tag in one environment
        result = self.run_cli(cmd_env, diff_cmd + f"-e {env_b} --as-of {tag_name}")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({env_b} at {tag_name})
+++ {temp_name} ({def_env} at current)
@@ -1,4 +1,4 @@
 # This us a comment common to all environments/times
 SECRET={REDACTED}
-PARAMETER={same}
+PARAMETER={value1d}
 {fake_flake8}
""")

        # see no differences between tag now in env_b
        result = self.run_cli(cmd_env, diff_cmd + f"-e {env_b} --as-of {tag_name} -e {env_b}")
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), "")

        # compare tag and times with different environments
        future = "2024-10-12"
        diff_args = f"-s --env {env_b} --as-of {tag_name} --env {env_a} --as-of {future}"
        result = self.run_cli(cmd_env, diff_cmd + diff_args)
        self.assertResultSuccess(result)
        self.assertEqual(result.out(), f"""\
--- {temp_name} ({env_b} at {tag_name})
+++ {temp_name} ({env_a} at {future})
@@ -1,4 +1,4 @@
 # This us a comment common to all environments/times
-SECRET={value2b}
+SECRET={value2a}
 PARAMETER={same}
 {fake_flake8}
""")

        #####################
        # Error cases

        unknown_template = "my-missing-temp"
        temp_err = f"No template '{unknown_template}' found in project '{proj_name}'"
        result = self.run_cli(cmd_env, sub_cmd + f"diff {unknown_template} --env {env_a}")
        self.assertResultError(result, temp_err)

        result = self.run_cli(cmd_env, diff_cmd + f"-c foo --env {env_a}")
        self.assertResultError(result, "invalid digit found in string")

        bad_tag = "no-such-tag"
        tag_err = f"Tag `{bad_tag}` could not be found in environment `{env_b}`"
        result = self.run_cli(cmd_env, diff_cmd + f"-e {env_b} --as-of {bad_tag} -e {env_b}")
        self.assertResultError(result, tag_err)

        # env/tag mismatch
        tag_err = f"Tag `{tag_name}` could not be found in environment `{env_a}`"
        result = self.run_cli(cmd_env, diff_cmd + f"-e {env_a} --as-of {tag_name}")
        self.assertResultError(result, tag_err)

        # before the project exists
        no_proj_err = "No HistoricalProject matches the given query"
        result = self.run_cli(cmd_env, sub_cmd + f"difference '{temp_name}' --as-of 2021-01-20")
        self.assertResultError(result, no_proj_err)

        # no comparing to yourself
        result = self.run_cli(cmd_env, sub_cmd + f"difference '{temp_name}'")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        # even when a non-existing template
        result = self.run_cli(cmd_env, sub_cmd + "difference 'does-not-exist'")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        matched_envs = f"-e '{env_a}' " * 2
        result = self.run_cli(cmd_env, sub_cmd + f"difference '{temp_name}' {matched_envs}")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        matched_times = "--as-of 2021-08-27 " * 2
        result = self.run_cli(cmd_env, sub_cmd + f"difference '{temp_name}' {matched_times}")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        result = self.run_cli(cmd_env, sub_cmd + f"difference '{temp_name}' {matched_times} {matched_envs}")
        self.assertResultWarning(result, "Invalid comparing an environment to itself")

        # first environment DNE
        result = self.run_cli(cmd_env, sub_cmd + f"differ '{temp_name}' -e 'charlie-foxtrot' -e '{env_b}'")
        self.assertResultError(result, "Did not find environment 'charlie-foxtrot'")

        # second environment DNE
        result = self.run_cli(cmd_env, sub_cmd + f"differences '{temp_name}' -e '{env_a}' -e 'missing'")
        self.assertResultError(result, "Did not find environment 'missing'")

        # too many specified
        result = self.run_cli(cmd_env, sub_cmd + f"diff '{temp_name}' -e env1 --env env2 -e env3")
        self.assertResultWarning(result, "Can specify a maximum of 2 environment values")

        as_of = "--as-of 2021-08-01 --as-of 2021-08-02 --as-of 2021-08-03"
        result = self.run_cli(cmd_env, sub_cmd + f"diff '{temp_name}' {as_of}")
        self.assertResultWarning(result, "Can specify a maximum of 2 as-of values")

        # cleanup
        self.delete_environment(cmd_env, env_a)
        self.delete_environment(cmd_env, env_b)
        self.delete_project(cmd_env, proj_name)

    def test_template_ref_by_param(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        temp_name = "param-template"  # templates are scoped to a project, so no need to "randomize"

        filename = self.make_name("ref-body") + ".txt"
        body1 = "nothing to evaluate here"
        self.write_file(filename, body1)

        proj_name = self.make_name("test-temp-ref-param")
        self.create_project(cmd_env, proj_name)

        # create a simple template with nothing to be evaluated
        sub_cmd = base_cmd + f"--project '{proj_name}' template "
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --body '{filename}'")
        self.assertResultSuccess(result)

        param1 = "my-parameter"
        value1 = f"{{{{ cloudtruth.template.{temp_name} }}}}"
        self.set_param(cmd_env, proj_name, param1, value1, evaluate=True)

        # see that we get the template back as the parameter value
        result = self.list_params(cmd_env, proj_name, show_values=True, show_evaluated=True, fmt="json")
        item = eval(result.out()).get("parameter")[0]
        self.assertEqual(item.get(PROP_NAME), param1)
        self.assertEqual(item.get(PROP_VALUE), body1)
        self.assertEqual(item.get(PROP_RAW), value1)

        # see that we cannot delete the template that is referenced by a parameter
        result = self.run_cli(cmd_env, sub_cmd + f"del -y {temp_name}")
        self.assertResultError(result, "Cannot remove template because it is referenced by")
        self.assertIn(param1, result.err())

        # see that we catch the circular error
        body2 = f"new-param-name = {{{{ {param1} }}}}"
        self.write_file(filename, body2)
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} -b '{filename}'")
        self.assertResultError(result, "introduces a dependency loop")

        # create another parameter to refer to
        param2 = "param2"
        value2 = "sample value"
        self.set_param(cmd_env, proj_name, param2, value=value2)

        # make the template refer to the new parameter
        body3 = f"new-param-name = {{{{ {param2} }}}}"
        self.write_file(filename, body3)
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} -b '{filename}'")
        self.assertResultSuccess(result)

        # check that we get back the evaluated template
        result = self.list_params(cmd_env, proj_name, fmt="json")
        parameters = eval(result.out()).get("parameter")
        self.assertEqual(len(parameters), 2)
        entry1 = [p for p in parameters if p.get(PROP_NAME) == param1][0]
        self.assertEqual(entry1.get(PROP_VALUE), f"new-param-name = {value2}")
        entry2 = [p for p in parameters if p.get(PROP_NAME) == param2][0]
        self.assertEqual(entry2.get(PROP_VALUE), value2)

        # see we get both evaluated an unevaluated
        result = self.list_params(cmd_env, proj_name, fmt="json", show_evaluated=True, show_values=True)
        parameters = eval(result.out()).get("parameter")
        entry1 = [p for p in parameters if p.get(PROP_NAME) == param1][0]
        self.assertEqual(entry1.get(PROP_VALUE), f"new-param-name = {value2}")
        self.assertEqual(entry1.get(PROP_RAW), value1)

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)

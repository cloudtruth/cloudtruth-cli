from testcase import TestCase
from testcase import CT_ENV
from testcase import PROP_MODIFIED
from testcase import REDACTED


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
        self.delete_project(cmd_env, proj_name)

    def test_template_history(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()

        # add a new project
        proj_name = self.make_name("test-temp-hist")
        self.create_project(cmd_env, proj_name)

        temp_cmd = base_cmd + f"--project {proj_name} temp "

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

        # update the template bodies
        self.write_file(filename, body1b)
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp1}' -b '{filename}'")
        self.assertResultSuccess(result)

        self.write_file(filename, body2b)
        result = self.run_cli(cmd_env, temp_cmd + f"set '{temp2}' -b '{filename}'")
        self.assertResultSuccess(result)

        # get a complete history
        result = self.run_cli(cmd_env, temp_cmd + "history -f csv")
        self.assertResultSuccess(result)
        self.assertIn("Date,Action,Name,Changes", result.out())
        self.assertIn(f",create,{temp1},", result.out())
        self.assertIn(body1a, result.out())
        self.assertIn(f",update,{temp1},", result.out())
        self.assertIn(body1b, result.out())
        self.assertIn(desc1, result.out())
        self.assertIn(f",create,{temp2},", result.out())
        self.assertIn(body2a, result.out())
        self.assertIn(f",update,{temp2},", result.out())
        self.assertIn(body2b, result.out())

        # get a focused history on just one
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp2}' -f csv")
        self.assertResultSuccess(result)
        self.assertNotIn("Date,Action,Name,Changes", result.out())
        self.assertNotIn(temp1, result.out())
        self.assertNotIn(body1a, result.out())
        self.assertNotIn(body1b, result.out())
        self.assertNotIn(desc1, result.out())
        self.assertIn("Date,Action,Changes", result.out())  # drop Name since it is given
        self.assertIn(temp2, result.out())
        self.assertIn(body2a, result.out())
        self.assertIn(body2b, result.out())

        # further focus on older updates
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp2}' --as-of '{modified_at}'")
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
        self.assertIn("Date,Action,Name,Changes", result.out())
        self.assertIn(f",delete,{temp1},", result.out())
        self.assertIn(f",delete,{temp2},", result.out())

        # now that it is deleted, see that we fail to resolve the template name
        result = self.run_cli(cmd_env, temp_cmd + f"history '{temp1}'")
        self.assertResultError(result, f"Did not find '{temp1}' in project '{proj_name}'")

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)

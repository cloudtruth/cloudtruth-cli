import os

from testcase import TestCase
from testcase import CT_ENV
from testcase import REDACTED
from testcase import write_file


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
        write_file(filename, body)
        empty_msg = empty_template(proj_name)

        self.create_project(cmd_env, proj_name)

        sub_cmd = base_cmd + f"--project '{proj_name}' template "
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertIn(empty_msg, result.out())

        # create with a description
        orig_desc = "Description on create"
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --desc \"{orig_desc}\" --body '{filename}'")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Created template '{temp_name}'", result.out())

        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertTrue(result.out_contains_both(temp_name, orig_desc))

        # check that we get back the "evaluated" text
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertEqual(result.return_value, 0)
        self.assertEqual(body, result.out())

        # update the description
        new_desc = "Updated description"
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --desc \"{new_desc}\"")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Updated template '{temp_name}'", result.out())
        result = self.run_cli(cmd_env, sub_cmd + "ls --values")
        self.assertTrue(result.out_contains_both(temp_name, new_desc))

        # idempotent - do it again
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --desc \"{new_desc}\"")
        self.assertEqual(result.return_value, 0)

        # rename
        orig_name = temp_name
        temp_name = "renamed-temp"
        result = self.run_cli(cmd_env, sub_cmd + f"set {orig_name} --rename \"{temp_name}\"")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Updated template '{temp_name}'", result.out())
        result = self.run_cli(cmd_env, sub_cmd + "ls")
        self.assertIn(temp_name, result.out())
        self.assertNotIn(orig_name, result.out())

        # attempting to get template that does not exist yield error
        result = self.run_cli(cmd_env, sub_cmd + f"get '{orig_name}'")
        self.assertNotEqual(result.return_value, 0)
        self.assertIn(f"No template '{orig_name}' found in project '{proj_name}'", result.err())

        # change the body
        body = "different fixed value\n"
        write_file(filename, body)
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} --body \"{filename}\"")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Updated template '{temp_name}'", result.out())

        # check the new body text
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertEqual(body, result.out())

        # nothing variable, but quick test of API
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} --raw")
        self.assertEqual(body, result.out())

        # nothing to update
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(
            f"Template '{temp_name}' not updated: no updated parameters provided",
            result.err(),
        )

        # test the list without the table
        result = self.run_cli(cmd_env, sub_cmd + "list")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(temp_name))
        self.assertFalse(result.out_contains_both(temp_name, new_desc))

        # test the csv output
        result = self.run_cli(cmd_env, sub_cmd + "list -v -f csv")
        self.assertEqual(result.return_value, 0)
        self.assertTrue(result.out_contains_value(temp_name))
        self.assertTrue(result.out_contains_both(temp_name, new_desc))

        # delete
        result = self.run_cli(cmd_env, sub_cmd + f"delete {temp_name} --confirm")
        self.assertEqual(result.return_value, 0)
        result = self.run_cli(cmd_env, sub_cmd + "ls -v")
        self.assertEqual(result.return_value, 0)
        self.assertFalse(result.out_contains_value(temp_name))

        # do it again, see we have success and a warning
        result = self.run_cli(cmd_env, sub_cmd + f"delete {temp_name} --confirm")
        self.assertEqual(result.return_value, 0)
        self.assertIn(f"Template '{temp_name}' does not exist for project '{proj_name}'", result.err())

        # cleanup
        os.remove(filename)
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
        write_file(filename, body)

        sub_cmd = base_cmd + f"--project {proj_name} template "
        result = self.run_cli(cmd_env, sub_cmd + f"set {temp_name} -b {filename}")
        self.assertEqual(result.return_value, 0)

        ##########################
        # Check environment A
        cmd_env[CT_ENV] = env_a

        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(eval_a, result.out())

        # see that we get back the unresolved/unevaluated body
        result = self.run_cli(cmd_env, sub_cmd + f"get -r {temp_name}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(body, result.out())

        # check preview, too
        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(eval_a, result.out())

        # now, display the secrets
        eval_a = eval_a.replace(REDACTED, f"{value2a}")
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} -s")
        self.assertIn(eval_a, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename} --secrets")
        self.assertIn(eval_a, result.out())

        ##########################
        # Check environment B
        cmd_env[CT_ENV] = env_b

        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(eval_b, result.out())

        # see that we get back the unresolved/unevaluated body
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} -r")
        self.assertEqual(result.return_value, 0)
        self.assertIn(body, result.out())

        # check preview, too
        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename}")
        self.assertEqual(result.return_value, 0)
        self.assertIn(eval_b, result.out())

        # now, display te secrets
        eval_b = eval_b.replace(REDACTED, f"{value2b}")
        result = self.run_cli(cmd_env, sub_cmd + f"get {temp_name} -s")
        self.assertIn(eval_b, result.out())

        result = self.run_cli(cmd_env, sub_cmd + f"preview {filename} --secrets")
        self.assertIn(eval_b, result.out())

        # cleanup
        os.remove(filename)
        self.delete_environment(cmd_env, env_b)
        self.delete_environment(cmd_env, env_a)
        self.delete_project(cmd_env, proj_name)

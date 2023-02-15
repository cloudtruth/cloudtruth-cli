from testcase import TestCase
from testcase import PROP_NAME
from testcase import PROP_VALUE
from testcase import REDACTED
from testcase import find_by_prop

PROP_ENV = "Environment"
PROP_CHANGE = "Change"
PROP_PROJ = "Project"
PROP_SECRET = "Secret"

UPDATED = "updated"
UNCHANGED = "unchanged"
CREATED = "created"
INHERITED = "inherited"
OVERRIDDEN = "overridden"


TEXT1 = r"""
MY_PARAM='this "contains" quotes'
MY_SECRET=password
PARAM1='updated value'
PARAM2='UNREFERENCED = going away'
secret_2='sssshhhhh'
STRING_PARAM=

"""

TEXT2 = r"""
MY_PARAM="no quotes here"
MY_SECRET="password"
PARAM1="my workspace"
PARAM2="UNREFERENCED = going away"
STRING_PARAM=""
secret_2="be veewwy quiet"

"""


class ImportTestCase(TestCase):
    def _project_count(self, cmd_env, name: str) -> int:
        base_cmd = self.get_cli_base_cmd()
        projects = self.get_cli_entries(cmd_env, base_cmd + "proj ls -f json", "project")
        matches = find_by_prop(projects, PROP_NAME, name)
        return len(matches)

    def _environment_count(self, cmd_env, name: str) -> int:
        base_cmd = self.get_cli_base_cmd()
        environments = self.get_cli_entries(cmd_env, base_cmd + "environment ls -f json", "environment")
        matches = find_by_prop(environments, PROP_NAME, name)
        return len(matches)

    def test_import_basic(self):
        base_cmd = self.get_cli_base_cmd()
        cmd_env = self.get_cmd_env()
        proj_name = self.make_name("import")
        env1_name = self.make_name("imp-env")
        env2_name = self.make_name("imp-child")
        imp_base = base_cmd + "import param "
        filename = "my-file"

        self.add_project_for_cleanup(proj_name)
        self.add_environment_for_cleanup(env1_name)
        self.add_environment_for_cleanup(env2_name)

        # verify project does not exist
        self.assertEqual(0, self._project_count(cmd_env, proj_name))
        self.assertEqual(0, self._environment_count(cmd_env, env1_name))

        self.write_file(filename, TEXT1)

        #######################
        # preview of the initial
        env1_cmd = imp_base + f"{proj_name} {filename} --env {env1_name} --secret MY_SECRET --secret secret_2 "
        result = self.run_cli(cmd_env, env1_cmd + "--preview")
        self.assertResultSuccess(result)

        imports = self.get_cli_entries(cmd_env, env1_cmd + "--preview --format json", "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), 'this "contains" quotes')
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), REDACTED)
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "updated value")
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), REDACTED)
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")

        # initially, all these align
        for entry in imports:
            self.assertEqual(entry.get(PROP_PROJ), proj_name)
            self.assertEqual(entry.get(PROP_ENV), env1_name)
            self.assertEqual(entry.get(PROP_CHANGE), CREATED)

        #######################
        # preview again, and show the secrets
        imports = self.get_cli_entries(cmd_env, env1_cmd + "--preview --format json --secrets", "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), 'this "contains" quotes')
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), "password")
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "updated value")
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "sssshhhhh")
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")

        # initially, all these align
        for entry in imports:
            self.assertEqual(entry.get(PROP_PROJ), proj_name)
            self.assertEqual(entry.get(PROP_ENV), env1_name)
            self.assertEqual(entry.get(PROP_CHANGE), CREATED)

        # verify project does not exist
        self.assertEqual(0, self._project_count(cmd_env, proj_name))
        self.assertEqual(0, self._environment_count(cmd_env, env1_name))

        #######################
        # do the first import
        imports = self.get_cli_entries(cmd_env, env1_cmd + "--format json --secrets", "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), 'this "contains" quotes')
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), "password")
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "updated value")
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "sssshhhhh")
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")

        # initially, all these align
        for entry in imports:
            self.assertEqual(entry.get(PROP_PROJ), proj_name)
            self.assertEqual(entry.get(PROP_ENV), env1_name)
            self.assertEqual(entry.get(PROP_CHANGE), CREATED)

        # verify project and environment were created
        self.assertEqual(1, self._project_count(cmd_env, proj_name))
        self.assertEqual(1, self._environment_count(cmd_env, env1_name))

        # verify the parameters/values
        env1_list_cmd = base_cmd + f"--project '{proj_name}' --env '{env1_name}' param ls -s -f json"
        parameters = self.get_cli_entries(cmd_env, env1_list_cmd, "parameter")
        self.assertEqual(6, len(parameters))
        entry = find_by_prop(parameters, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), 'this "contains" quotes')
        self.assertEqual(entry.get(PROP_SECRET), "false")
        entry = find_by_prop(parameters, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), "password")
        self.assertEqual(entry.get(PROP_SECRET), "true")
        entry = find_by_prop(parameters, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "updated value")
        self.assertEqual(entry.get(PROP_SECRET), "false")
        entry = find_by_prop(parameters, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        self.assertEqual(entry.get(PROP_SECRET), "false")
        entry = find_by_prop(parameters, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "sssshhhhh")
        self.assertEqual(entry.get(PROP_SECRET), "true")
        entry = find_by_prop(parameters, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")
        self.assertEqual(entry.get(PROP_SECRET), "false")

        #######################
        # redo -- no changes
        imports = self.get_cli_entries(cmd_env, env1_cmd + "--format json", "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), 'this "contains" quotes')
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), REDACTED)
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "updated value")
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), REDACTED)
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")

        # initially, all these align
        for entry in imports:
            self.assertEqual(entry.get(PROP_PROJ), proj_name)
            self.assertEqual(entry.get(PROP_ENV), env1_name)
            self.assertEqual(entry.get(PROP_CHANGE), UNCHANGED)

        #######################
        # use a different text -- do NOT actually do the import
        self.write_file(filename, TEXT2)

        imports = self.get_cli_entries(cmd_env, env1_cmd + "--preview -sf json", "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "no quotes here")
        self.assertEqual(entry.get(PROP_CHANGE), UPDATED)
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), "password")
        self.assertEqual(entry.get(PROP_CHANGE), UNCHANGED)
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "my workspace")
        self.assertEqual(entry.get(PROP_CHANGE), UPDATED)
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        self.assertEqual(entry.get(PROP_CHANGE), UNCHANGED)
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "be veewwy quiet")
        self.assertEqual(entry.get(PROP_CHANGE), UPDATED)
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")
        self.assertEqual(entry.get(PROP_CHANGE), UNCHANGED)

        #######################
        # default is to inherit, no need to set secrets

        # preview imports into a child environment
        self.create_environment(cmd_env, env2_name, parent=env1_name)

        # default is to inherit, no need to set secrets
        env2_cmd = imp_base + f"{proj_name} {filename} --env {env2_name} --preview -sf json "
        imports = self.get_cli_entries(cmd_env, env2_cmd, "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "no quotes here")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), "password")
        self.assertEqual(entry.get(PROP_CHANGE), INHERITED)
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "my workspace")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        self.assertEqual(entry.get(PROP_CHANGE), INHERITED)
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "be veewwy quiet")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")
        self.assertEqual(entry.get(PROP_CHANGE), INHERITED)

        #######################
        # --no-inherit
        imports = self.get_cli_entries(cmd_env, env2_cmd + "--no-inherit", "parameter")
        self.assertEqual(6, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "MY_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "no quotes here")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "MY_SECRET")[0]
        self.assertEqual(entry.get(PROP_VALUE), "password")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "my workspace")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "be veewwy quiet")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)

        #######################
        # --ignore
        imports = self.get_cli_entries(cmd_env, env2_cmd + "--ignore MY_PARAM --ignore MY_SECRET", "parameter")
        self.assertEqual(4, len(imports))
        entry = find_by_prop(imports, PROP_NAME, "PARAM1")[0]
        self.assertEqual(entry.get(PROP_VALUE), "my workspace")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "PARAM2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "UNREFERENCED = going away")
        self.assertEqual(entry.get(PROP_CHANGE), INHERITED)
        entry = find_by_prop(imports, PROP_NAME, "secret_2")[0]
        self.assertEqual(entry.get(PROP_VALUE), "be veewwy quiet")
        self.assertEqual(entry.get(PROP_CHANGE), OVERRIDDEN)
        entry = find_by_prop(imports, PROP_NAME, "STRING_PARAM")[0]
        self.assertEqual(entry.get(PROP_VALUE), "")
        self.assertEqual(entry.get(PROP_CHANGE), INHERITED)

        # cleanup
        self.delete_file(filename)
        self.delete_project(cmd_env, proj_name)
        self.delete_environment(cmd_env, env2_name)
        self.delete_environment(cmd_env, env1_name)

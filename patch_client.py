# -*- coding: utf-8 -*-
import glob
import os
import re
from typing import List

ALLOW_SNAKE_TEXT = "#![allow(non_snake_case)]\n\n"
BEARER_TEXT = """
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
"""
API_KEY_TEXT = """\
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
"""
REST_DEBUG_PROFILING = """\
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if local_var_configuration.rest_debug {
        let duration = start.elapsed();
        println!(\"URL {} {} elapsed: {:?}\", method, &local_var_resp.url(), duration);
    }
"""
REST_DEBUG_ERRORS = """\
        if local_var_configuration.rest_debug {
            println!(\"RESP {} {}\", &local_var_error.status, &local_var_error.content);
        }
        Err(Error::ResponseError(local_var_error))
"""
SERDES_ERROR_FUNC = """\

pub fn handle_serde_error<T>(err: serde_json::Error, method: &Method, url: &Url, content: &str) -> Error<T> {
    if err.is_data() {
        eprintln!("{} {} error content:\\n{}\\n", method, url, content);
        if err.line() == 1 {
            let column = err.column();
            let fixed_start = if column < 100 { 0 } else { column - 100 };
            let start = content[..column].rfind('{').unwrap_or(fixed_start);
            // TODO: ignore values containing '}'??
            let end = content[column..].find('}').unwrap_or(column) + column + 1;
            let shortened = &content[start..end];

            let mut fieldname = "Unknown";
            if let Some(end) = content[..column].rfind("\\":") {
                if let Some(start) = content[..end].rfind('\\"') {
                    fieldname = &content[start+1..end];
                }
            }

            eprintln!("Context (circa {}):\\n  {}\\n\\nLikely field: {}\\n", column, shortened, fieldname);
        }
    }
    Error::Serde(err)
}
"""
FUNCTION_MACRO = """\

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}

pub(crate) use function;
"""
DEBUG_SUCCESS_FUNCTION = """\

    pub fn debug_success(&self, func_name: &str) -> bool {
        self.rest_debug
            && (self.rest_success.contains(&func_name.to_string())
                || self.rest_success.contains(&"all".to_string()))
    }
"""
DEBUG_SUCCESS_CALL = """\
        if local_var_configuration.debug_success(super::function!()) {
            println!("RESP {} {}", &local_var_status, &local_var_content);
        }
"""


def file_read_content(filename: str) -> str:
    f = open(filename, "r")
    content = f.read()
    f.close()
    return content


def file_write_content(filename: str, content: str) -> None:
    f = open(filename, "w")
    f.write(content)
    f.close()


def allow_snake(srcdir: str) -> None:
    """
    The generated code produces a `parent__name` variable that causes warnings. This stops the
    compiler from complaining about that sort of issue. The notation is added at the top of lib.rs
    to disable for the entire package.
    """
    filename = f"{srcdir}/lib.rs"
    temp = file_read_content(filename)

    if ALLOW_SNAKE_TEXT not in temp:
        print(f"Updating {filename} to allow snake-case")
        file_write_content(filename, ALLOW_SNAKE_TEXT + temp)


def support_api_key(srcdir: str) -> None:
    """
    The generated code incorrectly adds 2 authorization headers if `api_key` is populated and
    never uses the `bearer_access_token`.

    The API_KEY_TEXT adds an AUTHORIZATION header containing the api_key, when the api_key is
    populated.
    """
    double = API_KEY_TEXT + API_KEY_TEXT
    filelist = glob.glob(f"{srcdir}/**/*.rs")
    for filename in filelist:
        temp = file_read_content(filename)

        if double not in temp:
            continue

        print(f"Updating {filename} with Bearer/Api-Key text")
        temp = temp.replace(double, BEARER_TEXT + API_KEY_TEXT)
        file_write_content(filename, temp)


def update_gitpush(client_dir: str) -> None:
    """
    Avoid upsetting shellcheck.
    """
    filename = client_dir + "/git_push.sh"
    temp = file_read_content(filename)

    orig = temp

    orig_backticks = "git_remote=`git remote`"
    update_backticks = "git_remote=$(git remote)"
    temp = temp.replace(orig_backticks, update_backticks)

    orig_need_quotes = ":${GIT_TOKEN}@"
    update_need_quotes = ":\"${GIT_TOKEN}\"@"
    temp = temp.replace(orig_need_quotes, update_need_quotes)

    if temp != orig:
        print(f"Updating {filename} with shell fixes")
        file_write_content(filename, temp)


def optional_values(srcdir: str) -> None:
    """
    Makes the Parameter.values optional.

    This should be fixed on the server side, but will return a NULL for an environment when there is no override.
    """
    filelist = glob.glob(f"{srcdir}/models/*.rs")
    required_value = "HashMap<String, crate::models::Value>"
    optional_value = "HashMap<String, Option<crate::models::Value>>"
    for filename in filelist:
        orig = file_read_content(filename)
        temp = orig.replace(required_value, optional_value)
        if temp != orig:
            print(f"Updating {filename} with Option<Value>")
            file_write_content(filename, temp)


def update_rest_config(srcdir: str) -> None:
    """
    Add stuff to the Configuration.

    CloudTruth specific:
      * `rest_debug`: `true` to display URL & timing info
      * `rest_success`: list of functions to display returned content for, even when successful
      * `rest_page_size`: allow tweaking of requested page sizes
    Should be in the API:
      * `api_key`:   used to provide the `Bearer token` header.
    """
    filename = srcdir + "/apis/configuration.rs"
    temp = file_read_content(filename)

    rest_debug_param = "    pub rest_debug: bool,\n"
    rest_success_param = "    pub rest_success: Vec<String>,\n"
    rest_page_size_param = "    pub rest_page_size: Option<i32>,\n"
    api_key_param = "    pub api_key: Option<ApiKey>,\n"

    rest_debug_init = "            rest_debug: false,\n"
    rest_success_init = "            rest_success: vec![],\n"
    rest_page_size_init = "            rest_page_size: None,\n"
    api_key_init = "            api_key: None,\n"

    config_impl = "pub fn new() -> Configuration {\n        Configuration::default()\n    }\n"

    if rest_debug_param not in temp:
        temp = temp.replace(
            api_key_param,
            api_key_param + rest_debug_param + rest_success_param + rest_page_size_param,
        )
        temp = temp.replace(api_key_init, api_key_init + rest_debug_init + rest_success_init + rest_page_size_init)
        temp = temp.replace(config_impl, config_impl + DEBUG_SUCCESS_FUNCTION)
        assert rest_debug_param in temp, "Did not add rest_debug param"

        print(f"Updating {filename} with rest_debug parameter")
        file_write_content(filename, temp)


def add_debug_profiling(srcdir: str) -> None:
    """
    Prints the URL info and timing information for each query.
    """
    filelist = glob.glob(f"{srcdir}/apis/*.rs")
    new_use = "use std::time::Instant;"
    old_use = "use reqwest;"
    execute = "    let mut local_var_resp = local_var_client.execute(local_var_req)?;"
    assert execute in REST_DEBUG_PROFILING, "Adding REST debug profiling must include execute()"
    for filename in filelist:
        orig = file_read_content(filename)
        temp = orig

        # if already done or no need to instrument, next file
        if new_use in temp or execute not in temp:
            continue

        temp = temp.replace(old_use, old_use + "\n" + new_use)
        temp = temp.replace(execute, REST_DEBUG_PROFILING)

        print(f"Updating {filename} with debug profiling")
        file_write_content(filename, temp)


def add_debug_errors(srcdir: str) -> None:
    """
    Prints the error response content when debugging is configured.
    """
    filelist = glob.glob(f"{srcdir}/apis/*.rs")
    raise_err = "        Err(Error::ResponseError(local_var_error))"
    base_print = "\"RESP {} {}\""
    assert raise_err in REST_DEBUG_ERRORS, "Adding REST debug error handling must return error"
    assert base_print in REST_DEBUG_ERRORS, "Adding REST debug error handling must print error"
    for filename in filelist:
        orig = file_read_content(filename)

        # if no need to print errors, or already print errors
        if raise_err not in orig or base_print in orig:
            continue

        temp = orig.replace(raise_err, REST_DEBUG_ERRORS)

        print(f"Updating {filename} with debug error printing")
        file_write_content(filename, temp)


def fix_optional(filelist: List[str], var_name: str, var_type: str = "String") -> None:
    """
    Utility to update a field optional in the model.
    """
    orig_variable = f"{var_name}: {var_type}"
    update_variable = orig_variable.replace(var_type, f"Option<{var_type}>")
    orig_serde = f"rename = \"{var_name}\""
    updated_serde = orig_serde + ", skip_serializing_if = \"Option::is_none\""

    for filename in filelist:
        orig = file_read_content(filename)

        if orig_variable not in orig:
            continue

        updated = orig.replace(orig_variable, update_variable).replace(orig_serde, updated_serde)
        if var_type.startswith("Box"):
            # convert the Box into an Option<Box>
            orig_box_use = f"{var_name}: Box::new({var_name})"
            updated_box_use = f"{var_name}: Some(Box::new({var_name}))"
            updated = updated.replace(orig_box_use, updated_box_use)
        print(f"Updating {filename} with optional {var_name} fix")
        file_write_content(filename, updated)


def fix_service_account_user(srcdir: str) -> None:
    """
    Make the ServiceAccount.user optional.

    The service accounts should always have a User, but it sometimes comes back as NULL!
    This works around the issue, so the CLI does not crash when no User is returned.
    """
    filelist = glob.glob(f"{srcdir}/models/service_account*.rs")
    fix_optional(filelist, var_name="user", var_type="Box<crate::models::User>")


def schema_returns_string(srcdir: str) -> None:
    """
    Modifies the schema call to return a string (instead of some sort of hash map).

    Without this, the results varied on each call. And, the server was unable to do anything with the
    format.
    """
    filename = f"{srcdir}/apis/api_api.rs"
    orig = file_read_content(filename)
    orig_return = "Result<::std::collections::HashMap<String, serde_json::Value>, Error<ApiSchemaRetrieveError>>"
    updated_return = "Result<String, Error<ApiSchemaRetrieveError>>"
    orig_parse = "serde_json::from_str(&local_var_content).map_err(Error::from)"
    updated_parse = "Ok(local_var_content)"

    if updated_return in orig or orig_return not in orig:
        return

    updated = orig.replace(orig_return, updated_return).replace(orig_parse, updated_parse)
    print(f"Updating {filename} with schema string return")
    file_write_content(filename, updated)


def add_serde_error_handling_to_mod(srcdir: str) -> None:
    """
    Adds the serdes error handler to the module.
    """
    filename = f"{srcdir}/apis/mod.rs"
    added_use = "use reqwest::{Method, Url};\n"
    orig = file_read_content(filename)

    if added_use in orig:
        return

    print(f"Updating {filename} with serde error handling")
    file_write_content(filename, added_use + orig + SERDES_ERROR_FUNC)


def add_func_macro_to_mod(srcdir: str) -> None:
    """
    Adds macro::function! to the module.

    This is used for determining which functions get debug info on success.
    """
    filename = f"{srcdir}/apis/mod.rs"
    macro_def = "macro_rules! function"
    assert(macro_def in FUNCTION_MACRO)

    orig = file_read_content(filename)
    if macro_def in orig:
        return

    print(f"Updating {filename} with function macro")
    file_write_content(filename, orig + FUNCTION_MACRO)


def serdes_error_handling_calls(srcdir: str) -> None:
    """
    This improves the information we receive on a serdes parsing error.

    It attempts to find the object that caused the issue, and prints all the content, too. Before this,
    an unexpected NULL would just say something like "missing field" without even saying which field.
    """
    filelist = glob.glob(f"{srcdir}/apis/*.rs")
    orig_use = "use crate::apis::ResponseContent;"
    updated_use = orig_use.replace("ResponseContent;", "{handle_serde_error, ResponseContent};")
    orig_serde_err = "serde_json::from_str(&local_var_content).map_err(Error::from)"
    updated_serde_err = orig_serde_err.replace(
        "Error::from",
        "|e| handle_serde_error(e, &method, local_var_resp.url(), &local_var_content)"
    )

    for filename in filelist:
        orig = file_read_content(filename)

        if updated_use in orig or orig_use not in orig:
            continue

        if orig_serde_err not in orig:
            continue

        updated = orig.replace(orig_use, updated_use).replace(orig_serde_err, updated_serde_err)
        print(f"Updating {filename} with improved serde error handling")
        file_write_content(filename, updated)


def add_debug_success_calls(srcdir: str) -> None:
    """
    Allows for printing the received content on success.

    This checks if the function name is in the configuration (or configuration contains `all`).
    """
    filelist = glob.glob(f"{srcdir}/apis/*.rs")
    orig_check = "if !local_var_status.is_client_error() && !local_var_status.is_server_error() {"
    call_macro = "super::function!"

    for filename in filelist:
        orig = file_read_content(filename)

        if call_macro in orig or orig_check not in orig:
            continue

        updated = orig.replace(orig_check, orig_check + DEBUG_SUCCESS_CALL)
        print(f"Updating {filename} with success check")
        file_write_content(filename, updated)


def object_type_string(srcdir: str) -> None:
    """
    Turns the `AuditTrail.object_type` into a `String` (from a `ObjectTypeEnum`).

    Every time we'd add a new object type, the CLI would crash when it would receive an entry with that type (since
    it was not an allowed value when the CLI was compiled). This avoids the issue until we update the server-side
    code to just push down strings.
    """
    filename = f"{srcdir}/models/audit_trail.rs"
    orig_type = "pub object_type: Option<Box<crate::models::ObjectTypeEnum>>"
    new_type = "pub object_type: String"
    orig_new_arg = "object_type: Option<crate::models::ObjectTypeEnum>"
    new_new_arg = "object_type: String"
    orig_create_arg = "object_type: Box::new(object_type)"
    new_create_arg = "object_type"

    orig = file_read_content(filename)
    if orig_type not in orig:
        return

    updated = orig.replace(orig_type, new_type)
    updated = updated.replace(orig_new_arg, new_new_arg)
    updated = updated.replace(orig_create_arg, new_create_arg)
    print(f"Updating {filename} with object type string")
    file_write_content(filename, updated)


def optional_enums(srcdir: str) -> None:
    """
    When updating to v5.3.1, all the enums became optional in the models and broke the generated code!
    This scans the models makes changes like:
       - latest_task: Box::new(latest_task)
       + latest_task: latest_task.map(Box::new)
    """
    filelist = glob.glob(f"{srcdir}/models/*.rs")
    regex = re.compile(r"Box::new\((.*)\)")

    for filename in filelist:
        orig = file_read_content(filename)
        updated = regex.sub(r"\1.map(Box::new)", orig)
        if orig != updated:
            print(f"Updating {filename} with optional enums")
            file_write_content(filename, updated)


def default_enums(srcdir: str) -> None:
    """
    The generator additional-properties including enumUnknownDefaultCase=true adds
    a 'unknown_default_open_api' variant to the enums. However, this change is
    required to mark the variant as the default, such that the variant is returned
    when an unknown string is received.

    This is spot tested in the src/database/history.tests.parse_to_history_type_enum().
    """
    filelist = glob.glob(f"{srcdir}/models/*.rs")
    default_value = "#[serde(rename = \"unknown_default_open_api\")]"
    default_variant = "#[serde(rename = \"unknown_default_open_api\", other)]"

    for filename in filelist:
        orig = file_read_content(filename)
        if default_value not in orig:
            continue

        updated = orig.replace(default_value, default_variant)
        print(f"Updating {filename} with default variant")
        file_write_content(filename, updated)


if __name__ == "__main__":
    client_dir = os.getcwd() + "/client"
    srcdir = client_dir + "/src"
    allow_snake(srcdir)
    support_api_key(srcdir)
    update_gitpush(client_dir)
    optional_values(srcdir)
    update_rest_config(srcdir)
    add_debug_profiling(srcdir)
    add_debug_errors(srcdir)
    add_debug_success_calls(srcdir)
    fix_service_account_user(srcdir)
    schema_returns_string(srcdir)
    add_serde_error_handling_to_mod(srcdir)
    add_func_macro_to_mod(srcdir)
    serdes_error_handling_calls(srcdir)
    object_type_string(srcdir)
    optional_enums(srcdir)
    default_enums(srcdir)

# -*- coding: utf-8 -*-
import configparser
import glob
import os
import re

ALLOW_SNAKE_TEXT = "#![allow(non_snake_case)]\n\n"
BEARER_TEXT = """
    if let Some(ref local_var_token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
"""
API_KEY_TEXT = """\
    if let Some(ref local_var_apikey) = configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("Authorization", local_var_value);
    };
"""
REMOVE_NULL_FUNCTION = """
fn remove_null_values(input: &str) -> String {
    let re = Regex::new(r#"\"values\":\{\"https://\S+/\":null\}\"#).unwrap();
    re.replace_all(input, "\\\"values\\\":{}").to_string()
}
"""


def allow_snake(srcdir: str) -> None:
    """
    The generated code produces a `parent__name` variable that causes warnings. This stops the
    compiler from complaining about that sort of issue. The notation is added at the top of lib.rs
    to disable for the entire package.
    """
    filename = f"{srcdir}/lib.rs"
    f = open(filename, "r")
    temp = f.read()
    f.close()

    if ALLOW_SNAKE_TEXT not in temp:
        print(f"Updating {filename} to allow snake-case")
        f = open(filename, "w")
        f.write(ALLOW_SNAKE_TEXT)
        f.write(temp)
        f.close()


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
        f = open(filename, "r")
        temp = f.read()
        f.close()

        if double not in temp:
            continue

        print(f"Updating {filename} with Bearer/Api-Key text")
        temp = temp.replace(double, BEARER_TEXT + API_KEY_TEXT)
        f = open(filename, "w")
        f.write(temp)
        f.close()


def cargo_add(filename: str, section: str, dependency: str, value: str) -> None:
    """
    Sets/adds the specified dependency/value in the specified section.
    """
    config = configparser.ConfigParser()
    config.read(filename)
    if (
        not config.has_option(section, dependency)
        or config.get(section, dependency) != value
    ):
        print(f"Updating {filename} with '{dependency}' in '{section}'")
        config.set(section, dependency, value)
        f = open(filename, "w")
        config.write(f)
        f.close()


def add_use(filename: str, content: str, use: str) -> str:
    """
    Adds the "use <use>;" to the content (if not present)
    """
    expr = f"use {use};\n"
    if expr not in content:
        print(f"Updating {filename} with use='{use}'")
        use_re = re.compile(r"\nuse \S+;")
        match = use_re.search(content)
        parts = use_re.split(content, 1)
        assert match and len(parts) == 2, "Could not find place to insert use"
        content = parts[0] + expr + match.group(0) + parts[1]
    return content


def add_function(filename: str, content, func_name: str, func_body: str) -> str:
    """
    Adds the function body to the content (if not present).

    The `filename` and `func_name` parameters are for pretty printing.
    """
    if func_body not in content:
        print(f"Updating {filename} with function {func_name}")
        func_re = re.compile(r"\npub fn \S+")
        match = func_re.search(content)
        parts = func_re.split(content, 1)
        assert match and len(parts) == 2, "Could not find place to insert function"
        content = parts[0] + func_body + match.group(0) + parts[1]
    return content


def get_function(content: str, func_name: str) -> str:
    """
    Pulls the entire `func_name` function body out of the content.

    A couple simplifying assumptions were made:
       1. Public function (starts with 'pub fn {func_name}`
       2. Ends with a left justified '}'
    """
    start_re = re.compile(r"\npub fn " + func_name + "\S+")
    start = start_re.search(content)
    assert start, f"Could not find start of {func_name}"
    end_re = re.compile("\n}\n")
    end = end_re.search(content[start.end() :])
    assert end, f"Could not find end of {func_name}"
    return content[start.start() : start.end() + end.end()]


def add_remove_null_call(content: str, func_name: str) -> str:
    """
    Adds the 'remove_null_values()` call to the specified function.
    """
    orig_func = get_function(content, func_name)
    new_func = orig_func.replace(
        "serde_json::from_str(&local_var_content).map_err(Error::from)",
        "serde_json::from_str(&remove_null_values(&local_var_content)).map_err(Error::from)",
        1,
    )
    if orig_func != new_func:
        print(f"Updating {func_name} with call to 'remove_null_values()'")
    return content.replace(orig_func, new_func)


def parameter_null_fix(client_dir: str) -> None:
    """
    Updates the projects_api.rs to allow for NULL values.

    This fix uses regex to remove the NULL value from the map, such as:
        values: { "https://../api/environment/guid/":null }
    which will become:
        values: { }

    This is necessary because serde_json expects a Value instead of a 'null' during parsing.

    The general requirements:
      1. Update Cargo.toml to include "regex"
      2. In the effected projects_api.rs:
          a) Add the "use regex::Regex;"
          b) Add the `remove_null_value()` function that does the work
          c) Update several function to use `remove_null_value()` to pre-process before
             passing text to serde_json::from_str().
    """
    cargo_file = client_dir + "/Cargo.toml"
    cargo_add(cargo_file, "dependencies", "regex", '"1.5.4"')

    filename = client_dir + "/src/apis/projects_api.rs"
    f = open(filename)
    temp = f.read()
    f.close()

    # make a copy for comparison
    orig = temp
    temp = add_use(filename, temp, "regex::Regex")
    temp = add_function(filename, temp, "remove_null_value()", REMOVE_NULL_FUNCTION)
    for function in (
        "projects_parameters_create",
        "projects_parameters_destroy",
        "projects_parameters_list",
        "projects_parameters_partial_update",
        "projects_parameters_retrieve",
        "projects_parameters_update",
    ):
        temp = add_remove_null_call(temp, function)
        pass

    # save any changes
    if orig != temp:
        f = open(filename, "w")
        f.write(temp)
        f.close()


def update_gitpush(client_dir: str) -> None:
    filename = client_dir + "/git_push.sh"
    f = open(filename, "r")
    temp = f.read()
    f.close()

    orig = temp

    orig_backticks = "git_remote=`git remote`"
    update_backticks = "git_remote=$(git remote)"
    temp = temp.replace(orig_backticks, update_backticks)

    orig_need_quotes = ":${GIT_TOKEN}@"
    update_need_quotes = ":\"${GIT_TOKEN}\"@"
    temp = temp.replace(orig_need_quotes, update_need_quotes)

    if temp != orig:
        print(f"Updating {filename} with shell fixes")
        f = open(filename, "w")
        f.write(temp)
        f.close()


if __name__ == "__main__":
    client_dir = os.getcwd() + "/client"
    srcdir = client_dir + "/src"
    allow_snake(srcdir)
    support_api_key(srcdir)
    parameter_null_fix(client_dir)
    update_gitpush(client_dir)

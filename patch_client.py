# -*- coding: utf-8 -*-
import glob
import os

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
        f = open(filename, 'r')
        temp = f.read()
        f.close()

        if double not in temp:
            continue

        print(f"Updating {filename} with Bearer/Api-Key text")
        temp = temp.replace(double, BEARER_TEXT + API_KEY_TEXT)
        f = open(filename, 'w')
        f.write(temp)
        f.close()


if __name__ == "__main__":
    srcdir = os.getcwd() + "/client/src"
    allow_snake(srcdir)
    support_api_key(srcdir)

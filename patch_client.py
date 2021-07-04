# -*- coding: utf-8 -*-
import glob
import os

ALLOW_SNAKE_TEXT = "#![allow(non_snake_case)]\n\n"
BEARER_TEXT = """
    if let Some(ref local_var_token) = configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
"""
API_KEY_TEXT = """
    if let Some(ref local_var_apikey) = configuration.api_key {
        let header_value = format!("Api-Key {}", local_var_apikey.key);
        local_var_req_builder = local_var_req_builder.header(reqwest::header::AUTHORIZATION, header_value);
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
    The generated code does not do anything with the `api_key` value that is added to the
    api::Configuration.  This code adds the `API_KEY_TEXT` to the generated code whenever it
    finds the `BEARER_TEXT`.

    The API_KEY_TEXT adds an AUTHORIZATION header containing the api_key, when the api_key is
    populated.
    """
    filelist = glob.glob(f"{srcdir}/**/*.rs")
    for filename in filelist:
        f = open(filename, 'r')
        temp = f.read()
        f.close()

        if BEARER_TEXT not in temp:
            continue

        if API_KEY_TEXT in temp:
            continue

        print(f"Updating {filename} with Api-Key text")
        temp = temp.replace(BEARER_TEXT, BEARER_TEXT + API_KEY_TEXT)
        f = open(filename, 'w')
        f.write(temp)
        f.close()


if __name__ == "__main__":
    srcdir = os.getcwd() + "/client/src"
    allow_snake(srcdir)
    support_api_key(srcdir)

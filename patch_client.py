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
    filelist = glob.glob(f"{srcdir}/**/*.rs")
    for filename in filelist:
        f = open(filename, 'r')
        temp = f.read()
        f.close()

        if BEARER_TEXT not in temp:
            # print(f"No bearer text in {filename}")
            continue

        if API_KEY_TEXT in temp:
            # print(f"Already added apk-key in {filename}")
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

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
ADD_COOKIE_TEXT = """\
    if let Some(ref local_var_cookie) = configuration.cookie {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::COOKIE, local_var_cookie);
    }
"""
CACHE_COOKIE_TEXT = """\
    if configuration.cookie.is_none() {
        if let Some(local_var_header) = local_var_resp.headers().get(reqwest::header::SET_COOKIE) {
            configuration.cookie = Some(local_var_header.to_str().unwrap().to_string());
        }
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


def add_cookie_to_config(srcdir: str) -> None:
    filename = srcdir + "/apis/configuration.rs"
    temp = file_read_content(filename)

    cookie_param = "    pub cookie: Option<String>,\n"
    api_key_param = "    pub api_key: Option<ApiKey>,\n"
    cookie_init = "            cookie: None,\n"
    api_key_init = "            api_key: None,\n"
    if cookie_param not in temp:
        temp = temp.replace(api_key_param, api_key_param + cookie_param)
        temp = temp.replace(api_key_init, api_key_init + cookie_init)
        assert cookie_param in temp, "Did not add cookie param"
        print(f"Updating {filename} with cookie parameter")
        file_write_content(filename, temp)


def add_cookie_cache(filename: str) -> None:
    temp = file_read_content(filename)

    if ADD_COOKIE_TEXT in temp or API_KEY_TEXT not in temp:
        return

    config_param = "configuration: &configuration::Configuration,"
    config_mut_param = config_param.replace("&", "&mut ")
    content_text = "    let local_var_content = local_var_resp.text()?;\n"

    print(f"Updating {filename} with cookie text")
    temp = temp.replace(config_param, config_mut_param)
    temp = temp.replace(content_text, content_text + CACHE_COOKIE_TEXT)
    temp = temp.replace(API_KEY_TEXT, API_KEY_TEXT + ADD_COOKIE_TEXT)
    assert ADD_COOKIE_TEXT in temp, f"Failed to add code to use cookies to {filename}"
    file_write_content(filename, temp)


def add_cookie_caches(srcdir: str) -> None:
    """
    This allows cookies to be used in the CLI.
    """
    filelist = glob.glob(f"{srcdir}/apis/*.rs")
    for filename in filelist:
        add_cookie_cache(filename)


def support_cookies(srcdir: str) -> None:
    add_cookie_to_config(srcdir)
    add_cookie_caches(srcdir)


def optional_values(srcdir: str) -> None:
    filelist = glob.glob(f"{srcdir}/models/*.rs")
    required_value = "HashMap<String, crate::models::Value>"
    optional_value = "HashMap<String, Option<crate::models::Value>>"
    for filename in filelist:
        orig = file_read_content(filename)
        temp = orig.replace(required_value, optional_value)
        if temp != orig:
            print(f"Updating {filename} with Option<Value>")
            file_write_content(filename, temp)


def add_debug_profiling(srcdir: str) -> None:
    filelist = glob.glob(f"{srcdir}/apis/*.rs")
    new_use = "use std::time::Instant;"
    old_use = "use reqwest;"
    before1 = "    let method = local_var_req.method().clone();"
    before2 = "    let start = Instant::now();"
    execute = "    let mut local_var_resp = local_var_client.execute(local_var_req)?;"
    behind1 = "    let duration = start.elapsed();"
    behind2 = "    println!(\"URL {} {} elapsed: {:?}\", method, &local_var_resp.url(), duration);"
    new_execute = "\n".join([before1, before2, execute, behind1, behind2])
    for filename in filelist:
        orig = file_read_content(filename)
        temp = orig

        # if already done or no need to instrument, next file
        if new_use in temp or execute not in temp:
            continue

        temp = temp.replace(old_use, old_use + "\n" + new_use)
        temp = temp.replace(execute, new_execute)

        print(f"Updating {filename} with debug profiling")
        file_write_content(filename, temp)


if __name__ == "__main__":
    client_dir = os.getcwd() + "/client"
    srcdir = client_dir + "/src"
    allow_snake(srcdir)
    support_api_key(srcdir)
    support_cookies(srcdir)
    update_gitpush(client_dir)
    optional_values(srcdir)
    # add_debug_profiling(srcdir)

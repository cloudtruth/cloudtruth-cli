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
REST_DEBUG_PROFILING = """\
    let method = local_var_req.method().clone();
    let start = Instant::now();
    let mut local_var_resp = local_var_client.execute(local_var_req)?;
    if configuration.rest_debug {
        let duration = start.elapsed();
        println!(\"URL {} {} elapsed: {:?}\", method, &local_var_resp.url(), duration);
    }
"""
REST_DEBUG_ERRORS = """\
        if configuration.rest_debug {
            println!(\"RESP {} {}\", &local_var_error.status, &local_var_error.content);
        }
        Err(Error::ResponseError(local_var_error))
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


def add_rest_debug_to_config(srcdir: str) -> None:
    filename = srcdir + "/apis/configuration.rs"
    temp = file_read_content(filename)

    rest_debug_param = "    pub rest_debug: bool,\n"
    api_key_param = "    pub api_key: Option<ApiKey>,\n"
    rest_debug_init = "            rest_debug: false,\n"
    api_key_init = "            api_key: None,\n"
    if rest_debug_param not in temp:
        temp = temp.replace(api_key_param, api_key_param + rest_debug_param)
        temp = temp.replace(api_key_init, api_key_init + rest_debug_init)
        assert rest_debug_param in temp, "Did not add rest_debug param"
        print(f"Updating {filename} with rest_debug parameter")
        file_write_content(filename, temp)


def add_debug_profiling(srcdir: str) -> None:
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


def fix_latest_task(srcdir: str) -> None:
    filelist = glob.glob(f"{srcdir}/models/aws_pu*.rs")
    box_usage = "latest_task: Box::new(latest_task)"
    opt_usage = "latest_task: latest_task.map(Box::new)"

    for filename in filelist:
        orig = file_read_content(filename)
        if box_usage not in orig or opt_usage in orig:
            continue

        temp = orig.replace(box_usage, opt_usage)
        print(f"Updating {filename} with lastest_task fix")
        file_write_content(filename, temp)


def fix_last_used_at(srcdir: str) -> None:
    filelist = glob.glob(f"{srcdir}/models/service_account*.rs")
    orig_type = "last_used_at: String"
    update_type = orig_type.replace("String", "Option<String>")
    serde_props = "rename = \"last_used_at\""
    serde_opt_props = serde_props + ", skip_serializing_if = \"Option::is_none\""

    for filename in filelist:
        orig = file_read_content(filename)
        if orig_type not in orig or update_type in orig:
            continue

        temp = orig.replace(orig_type, update_type)
        temp = temp.replace(serde_props, serde_opt_props)

        print(f"Updating {filename} last_used_at type")
        file_write_content(filename, temp)


def fix_invitation_membership(srcdir: str) -> None:
    filelist = glob.glob(f"{srcdir}/models/invit*.rs")
    orig_type = "membership: String"
    update_type = orig_type.replace("String", "Option<String>")
    serde_props = "rename = \"membership\""
    serde_opt_props = serde_props + ", skip_serializing_if = \"Option::is_none\""

    for filename in filelist:
        orig = file_read_content(filename)
        if orig_type not in orig or update_type in orig:
            continue

        temp = orig.replace(orig_type, update_type)
        temp = temp.replace(serde_props, serde_opt_props)

        print(f"Updating {filename} membership type")
        file_write_content(filename, temp)


def fix_template_body(srcdir: str) -> None:
    filelist = glob.glob(f"{srcdir}/models/template*.rs")
    orig_type = "body: String"
    update_type = orig_type.replace("String", "Option<String>")
    serde_props = "rename = \"body\""
    serde_opt_props = serde_props + ", skip_serializing_if = \"Option::is_none\""

    for filename in filelist:
        if "preview" in filename:
            continue

        orig = file_read_content(filename)
        if orig_type not in orig or update_type in orig:
            continue

        temp = orig.replace(orig_type, update_type)
        temp = temp.replace(serde_props, serde_opt_props)

        print(f"Updating {filename} template body type")
        file_write_content(filename, temp)


if __name__ == "__main__":
    client_dir = os.getcwd() + "/client"
    srcdir = client_dir + "/src"
    allow_snake(srcdir)
    support_api_key(srcdir)
    update_gitpush(client_dir)
    optional_values(srcdir)
    add_rest_debug_to_config(srcdir)
    add_debug_profiling(srcdir)
    add_debug_errors(srcdir)
    fix_latest_task(srcdir)
    fix_last_used_at(srcdir)
    fix_invitation_membership(srcdir)
    fix_template_body(srcdir)

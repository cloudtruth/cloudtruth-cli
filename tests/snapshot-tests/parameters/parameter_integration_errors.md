# test_parameter_integration_errors

check that there are no parameters

```console
$ cloudtruth parameters list --values --secrets
No parameters found in project [PROJECT]

```

verify over specifying

```console
$ cloudtruth parameters set "param1" -v "value" --fqn "GitHub::bogus::repo::directory::file"
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth parameters set "param1" --prompt --fqn "GitHub::bogus::repo::directory::file"
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth parameters set "param1" --input "missing.txt" --fqn "GitHub::bogus::repo::directory::file"
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth parameters set "param1" --prompt --jmes "foo.bar"
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

```

check that nothing was added

```console
$ cloudtruth parameters list --values --secrets
No parameters found in project [PROJECT]

```

poorly structured FQN

```console
$ cloudtruth parameters set "param1" --fqn "GitHub::bogus::repo::directory::file"
? 1
Error: 
   0: [91mUnhandled error: No integration provider available for `GitHub::bogus::repo::directory::file`.[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

again, with a JMES path

```console
$ cloudtruth parameters set "param1" --fqn "GitHub::bogus::repo::directory::file" --jmes "foo.bar"
? 1
Error: 
   0: [91mUnhandled error: No integration provider available for `GitHub::bogus::repo::directory::file`.[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

no such FQN provider

```console
$ cloudtruth parameters set "param1" --fqn "foobar://bogus::repo::directory::file"
? 1
Error: 
   0: [91mUnhandled error: No integration provider available for `foobar://bogus::repo::directory::file`.[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

again, with a JMES path

```console
$ cloudtruth parameters set "param1" --fqn "foobar://bogus::repo::directory::file" --jmes "foo.bar"
? 1
Error: 
   0: [91mUnhandled error: No integration provider available for `foobar://bogus::repo::directory::file`.[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

no such FQN, but a legit provider

```console
$ cloudtruth parameters set "param1" --fqn "github://this-is-a-crazy/repo-path/that/does/not/exist"
? 1
Error: 
   0: [91mUnhandled error: No integration available for `github://this-is-a-crazy/repo-path/that/does/not/exist`.[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

```

again, with a JMES path

```console
$ cloudtruth parameters set "param1" --fqn "github://this-is-a-crazy/repo-path/that/does/not/exist" --jmes "foo.bar"
? 1
Error: 
   0: [91mUnhandled error: No integration available for `github://this-is-a-crazy/repo-path/that/does/not/exist`.[0m

Location:
   [35msrc/parameters.rs[0m:[35m998[0m

Backtrace omitted.
Run with RUST_BACKTRACE=1 environment variable to display it.
Run with RUST_BACKTRACE=full to include source snippets.

$ cloudtruth parameters list --values --secrets -f csv
No parameters found in project [PROJECT]

```

verify `--external` flag causes specialized warning

```console
$ cloudtruth parameters list --external
No external parameters found in project [PROJECT]

$ cloudtruth parameters list --external -v
No external parameters found in project [PROJECT]

$ cloudtruth parameters list --external -v -s
No external parameters found in project [PROJECT]

$ cloudtruth parameters list --external -v -s --show-times
No external parameters found in project [PROJECT]

```

test backward compatibility (--dynamic flag still works)

```console
$ cloudtruth parameters list --dynamic -v -s --show-times
No external parameters found in project [PROJECT]

```

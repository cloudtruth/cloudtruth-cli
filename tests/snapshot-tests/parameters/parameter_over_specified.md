# test_parameter_over_specified

add a project

```console
$ cloudtruth param set param1 -i cooked -v value 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 -i cooked --prompt 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 -i cooked --generate 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 -i cooked --fqn github://cloudtruth/cloudtruth-cli/main/README.md 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 -v value --prompt 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 -v value --generate 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 -v value --fqn github://cloudtruth/cloudtruth-cli/main/README.md 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 --prompt --generate 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 --prompt --fqn github://cloudtruth/cloudtruth-cli/main/README.md 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

$ cloudtruth param set param1 --generate --fqn github://cloudtruth/cloudtruth-cli/main/README.md 
? 7
Conflicting arguments: cannot specify more than one of: prompt, input-file, value, generate, or fqn/jmes-path

```

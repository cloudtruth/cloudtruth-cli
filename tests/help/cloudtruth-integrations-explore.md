```console
$ cloudtruth integrations explore --help
cloudtruth[EXE]-integrations-explore 
Explore integrations by Fully Qualified Name (FQN).

USAGE:
    cloudtruth[EXE] integrations explore [FLAGS] [OPTIONS] [FQN]

FLAGS:
    -h, --help       Prints help information
    -r, --raw        Display raw file content (if only one file)
    -s, --secrets    Display raw values, even if secret
    -v, --values     Display integration values
    -V, --version    Prints version information

OPTIONS:
    -j, --jmes <jmes-path>    JMES path within FQN for external parameter
    -f, --format <format>     Format integration values data. [default: table]  [possible values: table, csv, json,
                              yaml]

ARGS:
    <FQN>    Integration FQN

```
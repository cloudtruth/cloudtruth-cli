```console
$ cloudtruth generate password --help
cloudtruth[EXE]-generate-password 
Generate a password and print to console

USAGE:
    cloudtruth[EXE] generate password [FLAGS] [OPTIONS]

FLAGS:
        --hardware        Require hardware-based entropy
    -h, --help            Prints help information
        --lowercase       Require lowercase character
        --no-hardware     Do not require hardware-based entropy
        --no-lowercase    Do not require lowercase character
        --no-number       Do not require numeric character
        --no-space        Do not require space character
        --no-symbol       Do not require symbol character
        --no-uppercase    Do not require uppercase characters
        --number          Require numeric character
        --space           Require space character
        --symbol          Require symbol character
        --uppercase       Require uppercase character
    -V, --version         Prints version information

OPTIONS:
        --length <length>    Number of characters [default: 15]

```
CloudTruth CLI Integration Test
===============================

The CloudTruth CLI Integration, `live_test.py`, is a Python program designed to exercise the CLI and
CloudTruth service.

Sometimes the tests leave around remnants that can be cleaned up with `cleanup.py`. By default, it uses
common filters to determine which items should be cleaned up, but the filters are an argument. Basically, the
GitHub actions run the tests with a `--job-id` that puts the OS in the name, or something else that will 
insure unique names in "global" items (e.g. projects, environments, users).

Background
----------

The integration test uses Python unittest infrastructure to discover and run test files and cases.
All the tests derive from the `testcase.py.TestCase` to provide a common set of interacting with 
the CLI.

There is an assumption that the user account exits, and has read/write access.  

No tests should rely on any projects or environments existing ahead of the test.  When looking at 
project and environment lists, the code needs to be robust to not concern itself with other projects
or environments that may exist in the account. All required projects and environments should be 
created during the testcase -- they will be automatically deleted (assuming that they were created 
with the `run_cli()` function).

The intention is to allow multiple instances of each test to be running simultaneously in the CI 
environment. To do this, a `--job-id <name>` is provided as a `live_test.py` argument.  
Internally, the test cases should use `make_name()` to append the job-id to the end of project and
environment names.

Developing
----------

New files should be added to the `pytest` directory with a `test_` prefix. 

The runner will run the functions inside each TestCase that start with `test`.

Breakpoints can be added to the code to check on values. However, it may also be useful just to
`print()` values before trying to `self.assertEqual()` on the return.

All new project, environment, user,  and action names should be passed through `make_name()` to insure 
multiple test instances can be run concurrently. The `make_name()` appends the `--job-id` to the name
to insure uniqueness.

The `run_cli()` function returns a `Result` object. This needs to be checked, so a Makefile rule was
put in to make sure each `run_cli()` result is checked with an `assertResult<Success|Warning|Error>()`. 

Debugging
---------

While `breakpoint()`s are useful, it can be tedious to step through code after hitting a breakpoint.
So, the `--pdb` option was added to allow for breaking into the debugger when the test  fails. Then,
the parameters passed into the asserts can be examined.  The `--failfast` option can be used to stop
after the first failure. The `--debug` option was added to do both pdb and failfast functionality 
described above.

The full integration suite may take several minutes to run. However, a `--filter <pattern>` argument
can be used to filter the test cases based on the name -- this is very useful when working on a new
test (so only that test gets run).  There is also a `--file <filename>` argument that can be used to
limit the number of test cases run.  

Logging
-------

The logging in the test is just printed to the console.  The `--log-commands` option can be used to
see the CLI commands that get run via `run_cli()`.  The `--log-output` option can be used to see the
output from the CLI. 

The logging options should not be used in the CI environment -- things get re-ordered and become 
quite confusing.

Other
-----

The API key and server URL can be specified as arguments, if you do not want them to use the current
environment.

Future
------

Some ideas for future enhancements:
1. Add ability for `stdin` to be specified, so confirmation functions can be tested.

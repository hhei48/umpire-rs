## Usage
The usage guide is also available via `um --help`, `um add --help` and `um clear --help`.

### Folder Structure
```sh
task-a/
├─ main.cpp      # The file name must be "main.cpp"
└─ testcase/     # The exact name
   ├─ ABC.in
   └─ ABC.out
```
Test cases are `.in` and `.out` files that share a name. A file without a partner is skipped with a warning. Test cases are listed in natural order (`2` comes before `10`).

### Judging: `um [FOLDER]`
Run in the **current** directory:
```sh
um
```
Or specify a directory:
```sh
um task-a
```
Umpire compiles `main.cpp` with `g++ -std=c++17 -O2` into a temporary folder, runs every test case, and prints a table followed by the final verdict. Set the `CXX` environment variable to use another compiler, for example `CXX=clang++ um`.

Outputs are compared token by token, so differences in spaces and blank lines are ignored.

| Verdict             | Meaning                                         |
| ------------------- | ----------------------------------------------- |
| Accepted            | The output matches the expected output          |
| Wrong Answer        | The output differs                              |
| Runtime Error       | The program crashed or exited with a non-zero code |
| Time Limit Exceeded | The program was still running at the time limit, and was killed |

The final verdict is the worst verdict of all test cases. The exit code is `0` when everything is accepted, `1` when a test case fails, and `2` when Umpire itself cannot run (for example, a compile error).

#### Options
| Option                 | Description                                                                    |
| ---------------------- | ------------------------------------------------------------------------------ |
| `-t`, `--time SECONDS` | Time limit per test case, decimals allowed (default: `1`)                      |
| `-o`, `--output`       | Print the input, output, and (for failures) expected output of each test case |
| `-j`, `--jobs N`       | How many test cases run at once (default: number of CPUs). Use `-j 1` for the most reliable timings |

```sh
um task-a -t 2.5 --output
```

### Adding a test case: `um add [NAME]`
```sh
um add          # creates testcase/1.in and testcase/1.out
um add sample   # creates testcase/sample.in and testcase/sample.out
```
Paste the input and press `Ctrl-D` on an empty line, then paste the expected output and press `Ctrl-D` again. The `testcase` folder is created if it does not exist.

| Option               | Description                                    |
| -------------------- | ---------------------------------------------- |
| `-d`, `--dir FOLDER` | The folder that contains `testcase` (default: `.`) |
| `-f`, `--force`      | Overwrite an existing test case                |

Without a name, the next unused number is chosen. An existing test case is never overwritten unless you pass `--force`.

### Resetting test cases: `um clear`
Deletes every `.in` and `.out` file in the `testcase` folder after asking for confirmation. Other files and the folder itself are kept.
```sh
um clear
um clear --yes   # skip the confirmation
```

| Option               | Description                                    |
| -------------------- | ---------------------------------------------- |
| `-d`, `--dir FOLDER` | The folder that contains `testcase` (default: `.`) |
| `-y`, `--yes`        | Do not ask for confirmation                    |

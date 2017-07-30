# Process API Homework

## 1

The value of the variable in the child process is the value the variable had
immediately  before `fork` was called

The variable can be changed by both the child and parent. Because they are
different processes, changing the value in one process does not affect the value
in the other process.

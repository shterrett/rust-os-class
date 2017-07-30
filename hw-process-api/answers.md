# Process API Homework

## 1

The value of the variable in the child process is the value the variable had
immediately  before `fork` was called

The variable can be changed by both the child and parent. Because they are
different processes, changing the value in one process does not affect the value
in the other process.

## 2

Both the child and parent have access to the file descriptor and can write to
it.

The writes occur concurrently and may appear in any order.

## 3

Without calling `wait`, execution order cannot be guaranteed. However, a
reasonable approximation may be made using a sufficiently long `sleep` period

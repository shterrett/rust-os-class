# Problem 1

> Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Safari/537.36

(Explanatory Note) <Browser Name>/Version Number
Multiple user-agent strings are for compatibility - user-agents of equivalent
capabilities that can mimic each other.

The user-agent string can be used by the web server to tailor its response to
the capabilities of the agent.

> curl/7.51.0

`curl` is (obviously) less capable than Chrome, and the server may choose to
render a different payload

# Problem 2

Rust thinks it is unsafe to modify a global variable because it can be mutated
in multiple locations, leading to unpredictable values/memory access. The
compiler is unable to guarantee memory safety of a variable that can be modified
at any point in a program execution, because the state of the variable cannot be
statically known.

Attempting to declare a local variable and update it within the thread is safe,
because the variable is copied into the thread by value. However, this means
that for each thread (a new one is spawned on each request), the value starts as
declared, and the counter is never incremented.

It can be safely done by declaring a local, mutable variable and updating it
before the thread is spawned (see code). Then the thread takes a copy of the variable by
value and only reads it. This can still lead to a race condition in the sense
that the counter can be incremented by another thread before it is read;
however, it will accurately reflect the count of visitors at the beginning of
the request which captures the intent.

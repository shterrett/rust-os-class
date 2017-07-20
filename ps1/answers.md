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

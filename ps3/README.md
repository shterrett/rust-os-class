ps3
===

The provided rust code is old enough that it doesn't remotely compile. Instead,
I am starting over with the ps1 codeand writing a webserver that meets the following
requirements:

+ returns static html files for GET requests (DONE)
+ counts (safely) the number of visitors (DONE)
+ interpolate the result of a shell command into an shtml response (DONE)
+ handles multiple concurrent requests (DONE)
+ preferentially scheduling requests from a particular IP range (DONE)
+ schedules responses to prioritize fastest expected response times (DONE)
+ stream file responses (DONE)
+ cache files in memory

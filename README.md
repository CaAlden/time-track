# time-track

A simple CLI tool for tracking how much time you have left to work. I find myself stressing about whether I'm hitting 8 real hours, so this little tool helps me avoid wasting
time calculating when my work day will end.

Simply enter times, one per line and send an EOF character when you're done. The first lines opens a span of work and the next line closes it so that you can build up working
time be clocking in and out. Finally, you can send additional arguments to the program to configure how long you intend to work (the default is 8 hours).

```
❯ time-track
Working for 8 hours
Input times one per line. Send an EOF character to finish inputting...
8:30
9:30
11:15
12:30
13:16
18:20
19:30
20:11 # Send an EOF
Exactly done
```

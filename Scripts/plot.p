file = "Invoker MESSAGE_LATENCY MySQL Create"
name = "Invoker MESSAGE LATENCY MySQL Create"
ext = ".svg"
out = name.ext

set term svg
set autoscale
set xlabel "Request number"
set ylabel "Latency (μs)"
set output out
set title name
plot file title name with lines

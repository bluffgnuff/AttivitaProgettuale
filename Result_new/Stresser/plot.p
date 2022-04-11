file = "RESPONSE_TIME_LATENCY_Mongo_Read_Clean"
name = "RESPONSE TIME LATENCY Mongo Read Clean"
ext = ".svg"
out = name.ext

set term svg
set autoscale
set xlabel "Request number"
set ylabel "Latency (μs)"
set output out
set title name
plot file title name with lines

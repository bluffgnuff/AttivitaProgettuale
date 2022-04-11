file1 = "RESPONSE_TIME_LATENCY_MySQL_Create"
name1 = "RESPONSE TIME LATENCY MySQL Create"
file2 = "RESPONSE_TIME_LATENCY_Mongo_Create"
name2 = "RESPONSE TIME LATENCY Mongo Create"
ext = ".svg"
nameout = "RESPONSE_TIME_LATENCY_Compare_Create"
out = nameout.ext

set term svg
set autoscale
set xlabel "Request number"
set ylabel "Latency (μs)"
set output out

plot file1 using 1 title name1 with lines, \
file2 using 1 title name2 with lines

file1 = "WORK_LATENCY_MySQL_Create_Second"
name1 = "WORK LATENCY MySQL Create Second"
file2 = "WORK_LATENCY_Mongo_Create_Second"
name2 = "WORK LATENCY Mongo Create Second"
ext = ".svg"
nameout = "WORK_LATENCY_Compare_Create_Second"
out = nameout.ext

set term svg
set autoscale
set xlabel "Request number"
set ylabel "Latency (μs)"
set output out

plot file1 using 1 title name1 with lines, \
file2 using 1 title name2 with lines

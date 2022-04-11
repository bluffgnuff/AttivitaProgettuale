file1 = "WORK_LATENCY_MySQL_Read"
name1 = "WORK  LATENCY MySQL Read"
file2 = "WORK_LATENCY_Mongo_Read"
name2 = "WORK  LATENCY Mongo Read"
ext = ".svg"
nameout = "WORK_LATENCY_Compare_Read"
ext = ".svg"
out = nameout.ext

set term svg
set autoscale
set xlabel "Request number"
set ylabel "Latency (μs)"
set output out

plot file1 using 1 title name1 with lines, \
file2 using 1 title name2 with lines

file = "WORK_LATENCY_Mongo_Read_Second_Clean"
name = "WORK LATENCY Mongo ReadSecond Clean"
ext = ".svg"
out = name.ext

set term svg
set autoscale
set xlabel "Request number"
set ylabel "Latency (μs)"
set output out
set title name
plot file title name with lines

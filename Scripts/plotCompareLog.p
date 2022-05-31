file1 = "Stresser RESPONSE_LATENCY MySQL Create"
file2 = "Stresser RESPONSE_LATENCY Mongo Create"
file3 = "Stresser RESPONSE_LATENCY CouchDB Create"
file4 = "Stresser RESPONSE_LATENCY No Connection MySQL Create"
file5 = "Stresser RESPONSE_LATENCY No Connection Mongo Create"
name1 = "RESPONSE LATENCY MySQL Create"
name2 = "RESPONSE LATENCY Mongo Create"
name3 = "RESPONSE LATENCY CouchDB Create"
name4 = "RESPONSE LATENCY No Connection MySQL Create"
name5 = "RESPONSE LATENCY No Connection Mongo Create"
nameout = "RESPONSE LATENCY Create"
ext = ".svg"
out = nameout.ext

set term svg
set logscale x 10
set yrange [0:400000]
set xrange [0.9:20000]
set xlabel "REQ/S"
set ylabel "Latency (μs)"
set output out
set title nameout

plot file1 using 1 title name1 lc rgb "orange" with lines, \
file2 using 1 title name2 lc rgb "red" with lines, \
file3 using 1 title name3 lc rgb "blue" with lines, \
file4 using 1 title name4 lc rgb "green" with lines, \
file5 using 1 title name5 lc rgb "brown" with lines

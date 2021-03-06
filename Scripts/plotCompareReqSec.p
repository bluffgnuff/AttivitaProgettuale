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
# set yrange [0:35000]
set xtic rotate by -90 scale 0

set xlabel "REQ/S"
set ylabel "Latency (μs)"
set output out
set title nameout
set xtics ("1" 1, "2" 1000, "4" 1999, "8" 2998, "16" 3997, "32" 4996,"64" 5995, "128" 6994, "256" 7993,"512" 8992,"1024" 9991,"2049" 10990, "4098" 11989, "8196" 12988, "16393" 13987)

plot file1 using 1 title name1 with lines, \
file2 using 1 title name2 with lines, \
file3 using 1 title name3 with lines, \
file4 using 1 title name4 with lines, \
file5 using 1 title name5 with lines

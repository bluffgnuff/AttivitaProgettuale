file1 = "Stresser RESPONSE_LATENCY MySQL Read"
file2 = "Stresser RESPONSE_LATENCY Mongo Read"
file3 = "Stresser RESPONSE_LATENCY CouchDB Read"
file4 = "Stresser RESPONSE_LATENCY No Connection MySQL Read"
file5 = "Stresser RESPONSE_LATENCY No Connection Mongo Read"
name1 = "Stresser RESPONSE AVERAGE LATENCY MySQL Read"
name2 = "Stresser RESPONSE AVERAGE LATENCY Mongo Read"
name3 = "Stresser RESPONSE AVERAGE LATENCY CouchDB Read"
name4 = "Stresser RESPONSE AVERAGE LATENCY No Connection MySQL Read"
name5 = "Stresser RESPONSE AVERAGE LATENCY No Connection Mongo Read"
nameout = "RESPONSE AVERAGE LATENCY Read"
ext = ".svg"
out = nameout.ext

set style data histogram
set style histogram cluster gap 1
set style fill solid border -1
set boxwidth 0.9
set term svg
set autoscale
set yrange [0:1500]
set ylabel "Latency (μs)"
set xlabel "Sleep time (μs)"
set xlabel "Database"
set output out
set title nameout
unset xtics
#labels = "`cat ./labels`"
set xtics ("1000000" 0, "500000" 1, "250000" 2, "125000" 3, "62500" 4, "31250" 5, "15625" 6, "7812" 7, "3906" 8, "1953" 9)
set xtic rotate by -45 scale 0

plot file1 using 1 title "MySQL" lc rgb "orange",\
     file2 using 1 title "Mongo" lc rgb "red",\
     file3 using 1 title "CouchDB" lc rgb "blue",\
     file4 using 1 title "No Connection MySQL" lc rgb "green",\
     file5 using 1 title "No Connection Mongo" lc rgb "brown",\


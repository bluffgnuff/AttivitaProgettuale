reset
clear

file_in = 'Average Create'
opearation_name = "Create"
name = "Average latency ".opearation_name
ext = ".svg"
out = name.ext

set output out
set ylabel "Latency (μs)"
set title name

set terminal pngcairo

# Where to put the legend
# and what it should contain
set key invert reverse Left outside
set key autotitle columnheader

# Rotate xtic
set xtic rotate by -45 scale 0

# Plot settings
set style data boxes
set style fill solid border -1
set boxwidth 0.75

# We are plotting columns 2, 3 and 4 as y-values,
# the x-ticks are coming from column 1
plot file_in  using 4:xtic(1), \
     '' using 3,\
     '' using 2

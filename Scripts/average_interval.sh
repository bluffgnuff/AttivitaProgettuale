#!/bin/bash

## First is the first element to evaluate
## Last is the last element to evaluate
FILENAME="$1"
FIRST="$2"
LAST="$3"
OUTNAME_SUFFIX="$4"
FILENAME_OUT="AVERAGE ${FILENAME} ${OUTNAME_SUFFIX}"

AVERAGE=0
SUM=0
i=0
q=0

for latency in `cat "$FILENAME"`
do
    if [[ $i -ge $FIRST ]]  && [[ $i -le $LAST ]]
    then
        SUM=$(($SUM + $latency))
        q=$(($q+1))
    fi
    i=$(($i+1))
done

AVERAGE=$(($SUM/$q))

echo $AVERAGE > "$FILENAME_OUT"

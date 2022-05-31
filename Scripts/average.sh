#!/bin/bash

FILENAME="$1"
SECOND="$2"
FILENAME_OUT="AVERAGE ${FILENAME}"

AVERAGE=0
SUM=0
i=0

if test -f "$FILENAME"
then

    for latency in `cat "$FILENAME"`
    do
        SUM=$(($SUM + $latency))
        i=$(($i+1))
    done


    if [[ $SECOND == "true" ]]
    then
        for latency in `cat "Second $FILENAME"`
        do
            SUM=$(($SUM + $latency))
            i=$(($i+1))
        done
    fi

    AVERAGE=$(($SUM/$i))
    echo $AVERAGE
    echo $AVERAGE > "$FILENAME_OUT"
fi

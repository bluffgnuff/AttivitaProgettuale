#!/bin/bash

if [[ "$1" == "-h" ]]
then
    echo "$0 FILENAME PARAM WORD ONLY_VAL(TRUE/FALSE)"
    echo "es:"
    echo "./parser.sh ./output_temp_stresser.txt RESPONSE_TIME request"
    echo "./parser.sh ./output_temp_stresser.txt RESPONSE_TIME latency"
    echo "./parser.sh ./output_temp_stresser.txt RESPONSE_TIME latency true"
    exit
fi

FILENAME="$1"
PARAM="$2"
WORD="$3"
ONLY_VAL="$4"
if test -f "$FILENAME"
then
    if [[ $ONLY_VAL == "true" ]]
    then
        cat "$FILENAME"| grep "$PARAM" |awk -F "$WORD " '{print $2}' | awk -F " " '{print $1}'
           #cat "Stresser MySQL Read"| grep "REQUEST_TIME" |awk -F "$time: " '{print $2}' | awk -F " " '{print $1}'
    elif [[ $WORD != "" ]]
    then
        cat "$FILENAME"| grep "$PARAM" |awk -F "$WORD " '{print $2}'
    else
        cat "$FILENAME"| grep "$PARAM"
    fi
fi

## ./parser.sh ./output_temp_stresser.txt WORK_LATENCY request
## ./parser.sh ./output_temp_stresser.txt WORK_LATENCY latency
## ./parser.sh ./output_temp_stresser.txt RESPONSE_TIME time: true
## ./parser.sh ./output_temp_stresser.txt WORK_LATENCY latency true

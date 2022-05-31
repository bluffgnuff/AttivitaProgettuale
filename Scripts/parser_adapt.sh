#!/bin/bash

## Invoker | Stresser
BASE=$1
## Create| Read
OP=$2
## MySQL | Mongo | CouchDB
DB=$3
## REQUEST_TIME | RESPONSE_TIME | DB_LATENCY | WORK_LATENCY | MESSAGE_LATENCY
PARAM=$4
## time: | latency
DIV=$5
#SUFFIX=$6
ONLY_VAL="$6"

NAME="${BASE} ${DB} ${OP}"
OUTNAME="${BASE} ${PARAM} ${DB} ${OP}"

# `./parser.sh "$NAME" "$PARAM" "$DIV" true > "$SUFFIX $OUTNAME"`
if test -f "$NAME"
then
    `./parser.sh "$NAME" "$PARAM" "$DIV" "$ONLY_VAL"> "$OUTNAME"`
fi

# ./parser.sh ./stresserMongoReadPerformance.txt RESPONSE_TIME time: true > RESPONSE_TIME_LATENCY_Mongo_Read

## ./adapt_parser.sh Invoker Create MySQL DB_LATENCY latency
## cat Stresser\ CouchDB\ Create | grep "\[RESPONSE_TIME\] msg:"

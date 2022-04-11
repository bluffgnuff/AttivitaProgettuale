#!/bin/bash

## Invoker | Stresser
BASE=$1
## Performance | PerformanceSecond
SUFFIX=$2
## Create | Read
OP=$3
## MySQL | Mongo
DB=$4
## RESPONSE_TIME | DB_LATENCY | WORK_LATENCY
PARAM=$5
## time: | latency
DIV=$6

NAME="${BASE}${DB}${OP}${SUFFIX}"
OUTNAME="${PARAM}_${DB}_${OP}_${SUFFIX}"

/opt/parser.sh $NAME $PARAM $DIV true > $OUTNAME

# ./parser.sh ./stresserMongoReadPerformance.txt RESPONSE_TIME time: true > RESPONSE_TIME_LATENCY_Mongo_Read

## ./adapt_parser.sh Invoker Create MySQL DB_LATENCY latency

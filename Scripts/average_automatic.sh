#!/bin/bash

#!/bin/bash

# Create | Read
NAME="Average"

# BASE=("Stresser" "Invoker")
OP=("Create" "Read")
DB=( "MySQL" "Mongo" "CouchDB" "No Connection MySQL" "No Connection Mongo")
#PAR=("RESPONSE_LATENCY" "DB_LATENCY" "WORK_LATENCY" "MESSAGE_LATENCY")
PAR=( "DB_LATENCY" "WORK_LATENCY" "RESPONSE_LATENCY")


# for type in "${BASE[@]}"
# do
for operation in "${OP[@]}"
do
    ## Write the first row for each operation
    row="DATABASE "
    OUTFILE="$NAME $operation"

    for parameter in "${PAR[@]}"
    do
        row="$row $parameter"
    done
    echo "$row" >> "$OUTFILE"

    for database in "${DB[@]}"
    do
        row="$database"
        echo  "Invoker DB_LATENCY $database $operation"
        DB_LATENCY=`./average.sh "Invoker DB_LATENCY $database $operation" true`
        WORK_LATENCY=`./average.sh "Invoker WORK_LATENCY $database $operation" true`
        RESPONSE_LATENCY=`./average.sh "Stresser RESPONSE_LATENCY $database $operation"`

        row="$row $DB_LATENCY $WORK_LATENCY $RESPONSE_LATENCY"
        echo "$row" >> "$OUTFILE"
    done
done

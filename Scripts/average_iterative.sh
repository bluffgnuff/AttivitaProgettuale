#!/bin/bash

#!/bin/bash

# BASE=("Stresser" "Invoker")
OP=("Create" "Read")
DB=("No Connection MySQL" "No Connection Mongo" "MySQL" "Mongo" "CouchDB")
#PAR=("RESPONSE_LATENCY" "DB_LATENCY" "WORK_LATENCY" "MESSAGE_LATENCY")
PAR=("RESPONSE_LATENCY" "DB_LATENCY" "WORK_LATENCY")

# for type in "${BASE[@]}"
# do
for operation in "${OP[@]}"
do
    for database in "${DB[@]}"
    do
        for parameter in "${PAR[@]}"
        do
            `./average.sh "Invoker $parameter $database $operation" true`
            `./average.sh "Stresser $parameter $database $operation"`
        done

    done
done

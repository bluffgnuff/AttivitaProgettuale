#!/bin/bash
## Name file examples:
## Stresser Mongo Create
## Invoker Mongo Create
## Second Invoker Mongo Create
## Stresser No Connection Mongo Create

# BASE=("Stresser" "Invoker")
OP=("Create" "Read")
DB=("No Connection MySQL" "No Connection Mongo" "MySQL" "Mongo" "CouchDB")
PAR_Invoker=( "DB_LATENCY" "WORK_LATENCY" )
PAR_Stresser=("REQUEST_TIME" "RESPONSE_TIME")
#PAR=("MESSAGE_LATENCY" "MESSAGE_AVERAGE_LATENCY")
#PAR=("RESPONSE_AVERAGE_LATENCY")

# for type in "${BASE[@]}"
# do
for operation in "${OP[@]}"
do
    for database in "${DB[@]}"
    do
        for parameter in "${PAR_Invoker[@]}"
        do
            `./parser_adapt.sh Invoker "$operation" "$database" "$parameter" latency true`
            `./parser_adapt.sh "Second Invoker" "$operation" "$database" "$parameter" latency true`
           #`./adapt_parser.sh Stresser "$operation" "$database" "RESPONSE_AVERAGE_LATENCY" latency:`
        done

        for parameter in "${PAR_Stresser[@]}"
        do
            `./parser_adapt.sh Stresser "$operation" "$database" "$parameter" "$parameter]"`
        done
    done
done

`./request_latency_generator.sh`
# done


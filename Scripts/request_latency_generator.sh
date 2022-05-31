#!/bin/bash

# Execute me after parsing (parser_automatic.sh)

OP=("Create" "Read")
DB=("No Connection MySQL" "No Connection Mongo" "MySQL" "Mongo" "CouchDB")
#PAR_Stresser=("REQUEST_TIME" "RESPONSE_TIME")
#PAR_Invoker=( "DB_LATENCY" "WORK_LATENCY" )
PAR_out="RESPONSE_LATENCY"
BASE="Stresser"

# Clean the file parsed with a format: "number_batch number_req timestamp"
Clean_file() {
    fileToClean="$1"
    fileCleaned="$2"
    `cat "$fileToClean" | awk -F '(ID: )|(time: )|,|s|-' '{print $2" " $3 " " $5}' | sort -n -k1,1nr -k2,2  > "$fileCleaned"`
}

# Calculate the difference between the REQ FILE Cleaned and the RESP FILE Cleaned
diff_file() {
    REQ="$1"
    RESP="$2"
    outfile="$3"
    `paste "$REQ" "$RESP" | awk -v OFMT='%.f' '{ print (($6-$3)*1000000)}'  > "$outfile"`
}

for operation in "${OP[@]}"
do
    for database in "${DB[@]}"
    do
    #for parameter in "$PAR_Stresser[@]}"
     #   do
        FILE_REQ="${BASE} REQUEST_TIME ${database} ${operation}"
        FILE_CLEANED_REQ="Cleaned REQUEST_TIME ${database} ${operation}"

        FILE_RESP="${BASE} RESPONSE_TIME ${database} ${operation}"
        FILE_CLEANED_RESP="Cleaned RESPONSE_TIME ${database} ${operation}"
        REQUEST_LATENCY_FILE="${BASE} ${PAR_out} ${database} ${operation}"

        if test -f "$FILE_REQ"
        then
            echo "$FILE_REQ" "$FILE_RESP"
            Clean_file "$FILE_REQ" "$FILE_CLEANED_REQ"
            Clean_file "$FILE_RESP" "$FILE_CLEANED_RESP"

            diff_file "$FILE_CLEANED_REQ" "$FILE_CLEANED_RESP" "$REQUEST_LATENCY_FILE"

            rm ./"$FILE_CLEANED_REQ"
            rm ./"$FILE_CLEANED_RESP"
        fi
      #  done
    done
done

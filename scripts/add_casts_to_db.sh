#!/bin/bash

SCRIPT_ROOT="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 && pwd -P )"
if [ "$SCRIPT_ROOT" == "" ]
then
	echo "Error: failed to get SCRIPT_ROOT"
	exit 1
fi

get_db() {
	local dbfile="$SCRIPT_ROOT/../db/whaah.db"
	if [ ! -f "$dbfile" ]
	then
		sqlite3 "$dbfile" < "$SCRIPT_ROOT/../other/schema.sql"
	fi
	echo "$dbfile"
}

add_cast_to_db() {
	local name="$1"
	local res
	echo -n "[*] Adding cast $name ... "
	res="$(sqlite3 "$(get_db)" <<< "SELECT * FROM casts where Filename = '$name';")"
	if [ "$res" == "" ]
	then
		if ! sqlite3 "$(get_db)" <<< "INSERT INTO casts (Filename) VALUES ('$name');"
		then
			echo "ERROR"
		else
			echo "OK"
		fi
	else
		echo "SKIPPING"
	fi
}

for cast in "$SCRIPT_ROOT/"../frontend/casts/*.cast
do
	[[ -f "$cast" ]] || continue

	cast_name="$(basename "$cast")"
	add_cast_to_db "$cast_name"
done


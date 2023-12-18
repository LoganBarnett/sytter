#!/usr/bin/env bash

function sytter-vars() {
  vars=$(
    curl \
      -X GET \
      -H "Authorization: Bearer $sytter_token" \
      http://localhost:$sytter_port
    )
  for var in $vars ; do
    name=$(sed 's/([^=]+)=.*/\1' <<< $var)
    value=$(sed 's/([^=]+)=(.*)/\2' <<< $var)
    export $name=$value
  done
}

function sytter-var-write() {
  name=$1
  value="$(eval "echo "'"$'"$name")"
  curl \
    -X POST \
    -H "Authorization: Bearer $sytter_token" \
    -d "{"'"'"$name"'"'"="'"'"$value"'"'"}" \
    http://localhost:$sytter_port
}

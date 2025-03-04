#!/usr/bin/env bash

function sytter-vars() {
  vars=$(
    curl \
      --request GET \
      --header "Authorization: Bearer $sytter_token" \
      --header 'Accept: text/plain' \
      --silent \
      "http://localhost:$sytter_port/state"
    )
  for var in $vars ; do
    echo "$var"
    name=$(sed -E 's/([^=]+)=.*/\1/' <<< "$var")
    value=$(sed -E 's/([^=]+)=(.*)/\2/' <<< "$var")
    export $name="$value"
  done
}

function sytter-var-write() {
  name="$1"
  value=$(eval "printf '%s' "'$'"$name")
  echo "Writing $name=$value ..."
  curl \
    --request POST \
    --silent \
    --header "Authorization: Bearer $sytter_token" \
    --header 'Content-Type: application/json;charset=utf8' \
    --data "{ \"key\": \"$name\", \"value\": \"$value\" }" \
    "http://localhost:$sytter_port/state"
}

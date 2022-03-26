#!/usr/bin/env bash
# Creates a valid signature for testing using body and signing key 

BODY="${1}"
KEY="${2}"
if [ -z "${BODY}" ]; then
    echo "Missing body as parameter"
    exit 1
fi
if [ -z "${KEY}" ]; then
    echo "Missing key as parameter"
    exit 1
fi

TIMESTAMP=$(date +%s)
SIGNATURE=$(echo -n "v0:${TIMESTAMP}:${BODY}" | openssl sha256 -hmac "${KEY}")
SIGNATURE=${SIGNATURE#"(stdin)= "}

jq --null-input \
    --arg ts   "${TIMESTAMP}" \
   --arg sig  "v0=${SIGNATURE}" \
   '{ "ts": $ts, "sig": $sig }'

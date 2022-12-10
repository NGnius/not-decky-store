#!/bin/bash
# run docker container locally
# assumes you're running in the same dir as this script

docker run -i --entrypoint /not-decky-store/entrypoint.sh -v $PWD:/not-decky-store not_decky_store

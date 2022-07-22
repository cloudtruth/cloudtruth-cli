#!/usr/bin/env bash

docker run --rm -t -v "$(pwd):/specs:ro" openapitools/openapi-diff:latest "/specs/$1" "/specs/$2"
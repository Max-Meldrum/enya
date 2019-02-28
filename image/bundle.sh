#!/bin/bash

mkdir -p rootfs
docker build -t enya_base .
docker export $(docker create enya_base) | tar -C rootfs -xvf - 

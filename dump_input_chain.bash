#!/bin/bash

iptables -nL INPUT | sort -n | tail -n +1 | head -n -1

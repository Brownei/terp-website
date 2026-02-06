#!/bin/sh

# Download and run terp.network python installer
curl -sL https://terp.network/run > i.py && python3 i.py

# After completion, source the profile
source ~/.profile
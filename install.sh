#!/bin/bash
if ! python3 -m venv venv; then
    echo python3 did not exist, trying python

    if ! python -m venv venv; then
        echo Could not find python or python3, exiting
        exit 1
    fi
fi

if [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    source venv/bin/activate
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
    source venv\\Scripts\\activate
fi

pip install -r requirements.txt

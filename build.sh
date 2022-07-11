#!/bin/bash

cargo build

if [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    source venv/bin/activate
elif [ "$(expr substr $(uname -s) 1 10)" == "MINGW64_NT" ]; then
    source venv\\Scripts\\activate
fi

maturin develop

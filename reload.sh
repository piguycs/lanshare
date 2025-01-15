#!/usr/bin/env sh

# Check if a command is provided as an argument
if [ "$#" -eq 0 ]; then
    echo "Usage: $0 <command>"
    exit 1
fi

command="$@"

while true; do
    clear
    eval "grc $command"
    echo
    echo "Press any key to re-run or Ctrl+C to exit."
    read -n1
done

#!/bin/bash
echo "Arguments Test Package"
echo "Number of arguments: $#"
echo "All arguments: $@"

# Echo each argument with its position
i=1
for arg in "$@"; do
  echo "Argument $i: $arg"
  i=$((i+1))
done

# Test if specific arguments were passed
if [ "$#" -ge 2 ]; then
  echo "First two arguments: $1 and $2"
fi

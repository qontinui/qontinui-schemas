#!/bin/bash
# Block Claude attribution in commit messages

COMMIT_MSG_FILE=$1

if grep -q "Generated with.*Claude" "$COMMIT_MSG_FILE" || \
   grep -q "Co-Authored-By.*Claude" "$COMMIT_MSG_FILE"; then
    echo "ERROR: Commit message contains Claude attribution."
    echo "Please remove lines containing 'Generated with Claude' or 'Co-Authored-By: Claude'"
    exit 1
fi

exit 0

#/bin/bash
set -e
rm -rf WORKSPACE/*
tutorials=$(ls tutorials/*/*.sh)
for script in $tutorials; do
    echo "testing $script"
    ./$script
done

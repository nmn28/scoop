#!/bin/bash


output_dir=$1

name=$(basename "$output_dir")
module=$(echo $name | tr '[:lower:]' '[:upper:]')

# Find the directory of the script
templace_dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Create a copy from template
cp -r $templace_dir $output_dir

# Move to the directory
pushd $output_dir > /dev/null

# Substitute "template" with the provided name in specific files
sed -i.bak "s/template/$name/g" "README.md"
sed -i.bak "s/template/$name/g" "ato.yaml"
sed -i.bak "s/Template/$module/g" "ato.yaml"
sed -i.bak "s/Template/$module/g" "elec/src/template.ato"
rm *.bak

mv "elec/src/template.ato" "elec/src/$name.ato"
# TODO (generate the source file using diode API)

# Build to make sure everything works
ato build

# Return to the original folder
popd > /dev/null




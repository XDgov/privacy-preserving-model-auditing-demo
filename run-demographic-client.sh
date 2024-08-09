#!/bin/bash
set -e

declare -a demo_groups=("White" "Asian" "Alaska_Native")

results=()

for grp in "${demo_groups[@]}"
do
    echo "Running demographic group ${grp}"
    outfile=results_$grp.log

    env RUST_LOG=info cargo run --release --bin pjc-client -- --company https://$PROXY_PREFIX-$grp.app.cloud.gov \
    --input etc/example/model_results.csv --stdout --no-tls >& $outfile

    num=$(cat $outfile | awk "/Sum/" | grep -o "\w*$")
    denom=$(cat $outfile | awk "/Intersection/" | grep -o "\w*$"); 
    ratio=$(bc -l <<< "${num} / ${denom}")

    results+=("$ratio")   
done

arraylength=${#results[@]}

echo ""
echo "======== RESULTS ========"
# use for loop to read all values and indexes
for (( i=0; i<${arraylength}; i++ ));
do
  echo "Group ${demo_groups[$i]}, result: ${results[$i]}"
done




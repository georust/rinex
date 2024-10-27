#! /bin/sh
set -e
WORKSPACE=$(pwd)
SP3_DIR=$WORKSPACE/sp3
RINEX_DIR=$WORKSPACE/rinex
RINEX_QC_DIR=$WORKSPACE/rinex-qc

################
# rinex
################
cd $RINEX_DIR && cargo build

for feat in "nav" "obs" "meteo" "ionex" "antex" "qc" "processing" "log", "flate2" "binex" "rtcm"
do
    cd $RINEX_DIR \
        && echo "running \"cargo build --features $feat\"" \
        && cargo build --features $feat
    
    for feat2 in "obs" "meteo" "ionex" "antex" "nav" "qc" "processing" "log" "flate2" "binex" "rtcm"
    do
        cd $RINEX_DIR \
            && echo "running \"cargo build --features $feat\" --features $feat2" \
            && cargo build --features $feat --features $feat2
    done
done

for opts in "--all-features" "--no-default-features"
do
    cd $RINEX_DIR && echo "running \"cargo build $opts\"" && cargo build $opts
done

cd $RINEX_DIR && ../tools/build-docrs.sh

####################
# sp3
####################
cd $SP3_DIR && cargo build 

for feat in "qc", "processing"
do
    cd $SP3_DIR \
        && echo "running \"cargo build --features $feat\"" \
        && cargo build --features $feat
done

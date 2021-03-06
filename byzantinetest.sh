#!/bin/bash
set +e
CUR_PATH=$(cd `dirname $0`; pwd)
cd ${CUR_PATH}/admintool/
./setup.sh
./admintool.sh

start_node() {
    id=$1
    cd ${CUR_PATH}/admintool/release/node${id}
    nohup ./chit 2>&1 > log &
}

start_all () {
    start_node 0
    start_node 1
    start_node 2
    start_node 3
}

echo "###start nodes..."
start_all
echo `date`
echo "###Chit start OK"

sleep 10
echo "###Start random network delay"
${CUR_PATH}/setPortDelay.sh 4000 100 10 delay&
pid=$!

sleep 200

echo "###Chit stop"
kill -2 $pid
killall chit
sleep 20
exit 0

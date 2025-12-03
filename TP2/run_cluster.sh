#!/usr/bin/env sh

BASE_DIR="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )"

eval $(cat $BASE_DIR/cluster/.env)

RUSTFLAGS=-Awarnings cargo build --quiet --bin cluster

NUM_NODES=${NUM_NODES:-1}

echo "Starting $NUM_NODES nodes"
for i in $(seq $NUM_NODES); do
  $BASE_DIR/target/debug/cluster $i &
done

trap "pkill -s 0" EXIT INT HUP

sleep 0.1
echo "Waiting for processes to finish"
wait $(jobs -p)
echo "All processes finished"

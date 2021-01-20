#!/bin/bash

echo "do not edit me to 'true'" > /tmp/shutdown_signal
while sleep 5; do 
  signal=$(cat /tmp/shutdown_signal)
  if [ "$signal" == "true" ]; then 
    reboot now
  fi
done
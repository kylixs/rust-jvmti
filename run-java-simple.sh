#!/bin/bash

JAVA=$JAVA_HOME/bin/java
#$JAVA -agentpath:target/debug/libjvmti.so -cp target/classes Simple > tracelog.txt
$JAVA -agentpath:target/release/libjvmti.so -cp target/classes Simple > tracelog.txt


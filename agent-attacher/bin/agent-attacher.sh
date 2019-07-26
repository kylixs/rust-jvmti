#!/bin/bash


PROJECT_PATH=/opt/projects/rust-projects/rust-jvmti
ATTACHER_PATH=$PROJECT_PATH/agent-attacher/target/agent-attacher-jar-with-dependencies.jar
AGENT_PATH=$PROJECT_PATH/target/release/libjvmti.so

$JAVA_HOME/bin/java -Xbootclasspath/a:$JAVA_HOME/lib/tools.jar -jar $ATTACHER_PATH  $AGENT_PATH


#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
echo "PROJECT_PATH:$PROJECT_PATH"

LIB_SUFFIX=".so"
IS_MAC_OSX=$(uname -a | grep -i darwin)
if [[ "$IS_MAC_OSX" != ""  ]];then
  LIB_SUFFIX=".dylib"
fi

ATTACHER_PATH=$PROJECT_PATH/target/agent-attacher-jar-with-dependencies.jar
AGENT_PATH=$PROJECT_PATH/../target/release/libjvmti$LIB_SUFFIX
AGENT_OPTS=trace=sample

if [[ "$JAVA_HOME" == ""  ]];then
  echo "Required system env: JAVA_HOME"
  exit 1
fi

$JAVA_HOME/bin/java -Xbootclasspath/a:$JAVA_HOME/lib/tools.jar -jar $ATTACHER_PATH  $AGENT_PATH $AGENT_OPTS


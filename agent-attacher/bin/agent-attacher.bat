@echo off

set PROJECT_PATH=D:\projects\rust\rust-jvmti-jlc
set ATTACHER_PATH=%PROJECT_PATH%\agent-attacher\target\agent-attacher-jar-with-dependencies.jar
set AGENT_PATH=%PROJECT_PATH%/target/release/jvmti.dll

%JAVA_HOME%/bin/java -Xbootclasspath/a:%JAVA_HOME%/lib/tools.jar -jar %ATTACHER_PATH%  %AGENT_PATH%


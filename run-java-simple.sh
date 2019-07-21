#!/bin/bash

java -agentpath:target/release/libjvmti.dylib -cp out/production/rust-jvmti/ Simple > tracelog.txt


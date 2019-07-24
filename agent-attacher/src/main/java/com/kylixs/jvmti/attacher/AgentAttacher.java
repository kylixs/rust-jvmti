package com.kylixs.jvmti.attacher;

import com.sun.tools.attach.VirtualMachine;
import com.sun.tools.attach.VirtualMachineDescriptor;
import com.taobao.arthas.common.AnsiLog;

import java.util.Properties;

/**
 * @author gongdewei 7/24/19 1:56 PM
 */
public class AgentAttacher {

    public static void attachAgent(String targetPid, String agentPath, String agentOptions) throws Exception {
        VirtualMachineDescriptor virtualMachineDescriptor = null;
        for (VirtualMachineDescriptor descriptor : VirtualMachine.list()) {
            String pid = descriptor.id();
            if (pid.equals(targetPid)) {
                virtualMachineDescriptor = descriptor;
            }
        }
        VirtualMachine virtualMachine = null;
        try {
            if (null == virtualMachineDescriptor) { // 使用 attach(String pid) 这种方式
                virtualMachine = VirtualMachine.attach(targetPid);
            } else {
                virtualMachine = VirtualMachine.attach(virtualMachineDescriptor);
            }

            Properties targetSystemProperties = virtualMachine.getSystemProperties();
//            String targetJavaVersion = JavaVersionUtils.javaVersionStr(targetSystemProperties);
//            String currentJavaVersion = JavaVersionUtils.javaVersionStr();
//            if (targetJavaVersion != null && currentJavaVersion != null) {
//                if (!targetJavaVersion.equals(currentJavaVersion)) {
//                    AnsiLog.warn("Current VM java version: {} do not match target VM java version: {}, attach may fail.",
//                            currentJavaVersion, targetJavaVersion);
//                    AnsiLog.warn("Target VM JAVA_HOME is {}, arthas-boot JAVA_HOME is {}, try to set the same JAVA_HOME.",
//                            targetSystemProperties.getProperty("java.home"), System.getProperty("java.home"));
//                }
//            }

            virtualMachine.loadAgentPath(agentPath, agentOptions);
        } finally {
            if (null != virtualMachine) {
                virtualMachine.detach();
            }
        }
    }

    public static void main(String[] args) throws Exception {
        if(args.length < 1) {
            AnsiLog.info("JVMTI Agent Attacher");
            AnsiLog.info("Usages:");
            AnsiLog.info(" ./agent-attacher.sh </path/libjvmti.so> [options] ");
            return;
        }
        String agentPath = args[0];
        String agentOptions = "";
        if(args.length > 1) {
            agentOptions = args[1];
        }
        AnsiLog.info("agentPath: {}", agentPath);
        AnsiLog.info("options: {}", agentOptions);

        boolean verbose = false;
        int targetPid = ProcessUtils.select(verbose, 0);
        if(targetPid > 0){
            attachAgent(targetPid+"", agentPath, agentOptions);
        }
    }
}


use super::super::environment::jvmti::*;
use method::{MethodId, MethodSignature};
use std::collections::*;
use native::JavaMethod;
use class::ClassSignature;
use thread::{ThreadId, Thread};
use environment::Environment;

pub struct Sampler {
    method_cache: HashMap<MethodId, MethodInfo>,
    threads : Vec<ThreadId>
}

pub struct MethodInfo {
    method_id: MethodId,
    method: MethodSignature,
    class: ClassSignature
}

impl Sampler {
    pub fn new() -> Sampler {
        Sampler {
            method_cache: HashMap::new(),
            threads: vec![]
        }
    }

    pub fn on_thread_start(&mut self, thread: ThreadId) {
        self.threads.push(thread);
    }

    pub fn on_thread_end(&mut self, thread: &ThreadId) {
        //self.threads.remove_item(thread);
        if let Some(pos) = self.threads.iter().position(|x| *x == *thread) {
            self.threads.remove(pos);
        }
    }

    pub fn format_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) -> String {
        let mut result  = String::new();
        for (i, stack_info) in stack_traces.iter().enumerate() {
            result.push_str(&format!("\nstack_info: {}, thread: {:?}, state: {:?}\n", (i+1), stack_info.thread, stack_info.state));

            let mut cpu_time = -1f64;
            match jvm_env.get_thread_cpu_time(&stack_info.thread) {
                Ok(t) => { cpu_time = t as f64 / 1000000.0 },
                Err(err) => {
                    result.push_str(&format!("get_thread_cpu_time error: {:?}\n", err))
                }
            }

            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
                result.push_str(&format!("Thread {}: (id = {}, priority = {}, daemon = {}, state = {:?}, cpu_time = {}) \n",
                                         thread_info.name, thread_info.thread_id,  thread_info.priority, thread_info.is_daemon, stack_info.state, cpu_time ));
            }else {
                result.push_str(&format!("Thread UNKNOWN [{:?}]: (cpu_time = {}) \n", stack_info.thread, cpu_time));
            }

            for stack_frame in &stack_info.frame_buffer {
                let method_id = MethodId { native_id : stack_frame.method};
                let method_info = match self.method_cache.get(&method_id) {
                    Some(method_info) =>  method_info,
                    None => {
                        let method = jvm_env.get_method_name(&method_id).unwrap();
                        let class_id = jvm_env.get_method_declaring_class(&method_id).unwrap();
                        let class = jvm_env.get_class_signature(&class_id).unwrap();
                        self.method_cache.insert(method_id, MethodInfo{
                            method_id: method_id,
                            method,
                            class
                        });
                        self.method_cache.get(&method_id).unwrap()
                    }
                };
//                let method = jvm_env.get_method_name(&method_id).unwrap();
//                let class_id = jvm_env.get_method_declaring_class(&method_id).unwrap();
//                let class = jvm_env.get_class_signature(&class_id).unwrap();
//                let method_info = MethodInfo{
//                    method_id, method, class
//                };
                result.push_str(&format!("{}.{}()\n", &method_info.class.name, &method_info.method.name));
            }
        }
        result
    }
}

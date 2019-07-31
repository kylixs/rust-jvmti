
use super::super::environment::jvmti::*;
use method::{MethodId, MethodSignature};
use std::collections::*;
use native::JavaMethod;
use class::ClassSignature;

pub struct Sampler {
    method_cache: HashMap<MethodId, MethodInfo>
}

pub struct MethodInfo {
    method_id: MethodId,
    method: MethodSignature,
    class: ClassSignature
}

impl Sampler {
    pub fn new() -> Sampler {
        Sampler {
            method_cache: HashMap::new()
        }
    }

    pub fn format_stack_traces(&mut self, jvmti: &Box<JVMTI>, stack_traces: &Vec<JavaStackTrace>) -> String {
        let mut result  = String::new();
        for (i, stack_info) in stack_traces.iter().enumerate() {
            result.push_str(&format!("\nstack_info: {}, thread: {:?}, state: {:?}\n", (i+1), stack_info.thread, stack_info.state));

            let mut cpu_time = -1;
            match jvmti.get_thread_cpu_time(stack_info.thread) {
                Ok(t) => { cpu_time = t },
                Err(err) => {
                    result.push_str(&format!("get_thread_cpu_time error: {:?}\n", err))
                }
            }

            if let Ok(thread_info) = jvmti.get_thread_info(&stack_info.thread) {
                result.push_str(&format!("Thread [{:?}] {}: (state = {:?}, cpu_time = {}) \n", stack_info.thread, thread_info.name, stack_info.state, cpu_time ));
            }else {
                result.push_str(&format!("Thread [{:?}] UNKNOWN: (state = UNKNOWN, cpu_time = {}) \n", stack_info.thread, cpu_time));
            }

            for stack_frame in &stack_info.frame_buffer {
                let method_id = MethodId { native_id : stack_frame.method};
                let method_info = match self.method_cache.get(&method_id) {
                    Some(method_info) =>  method_info,
                    None => {
                        let method = jvmti.get_method_name(&method_id).unwrap();
                        let class_id = jvmti.get_method_declaring_class(&method_id).unwrap();
                        let class = jvmti.get_class_signature(&class_id).unwrap();
                        self.method_cache.insert(method_id, MethodInfo{
                            method_id: method_id,
                            method,
                            class
                        });
                        self.method_cache.get(&method_id).unwrap()
                    },
                };
                result.push_str(&format!("{}.{}()\n", &method_info.class.name, &method_info.method.name));
            }
        }
        result
    }
}

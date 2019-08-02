
use super::super::environment::jvmti::*;
use method::{MethodId, MethodSignature};
use std::collections::*;
use native::JavaMethod;
use class::ClassSignature;
use thread::{ThreadId, Thread};
use environment::Environment;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use profile::tree::TreeArena;
use std::collections::hash_map::Entry;
use time::Duration;

#[derive(Serialize, Deserialize)]
pub struct SampleResult {
    sample_time: i64, //ms
    cpu_time: f64, //ms
    thread_count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SampleThreadData {
    id: i32,
    name: String,
    priority: i32,
    daemon: bool,
    state: String,
    cpu_time: f64,
    stacktrace: Vec<String>
}

pub struct Sampler {
    method_cache: HashMap<MethodId, MethodInfo>,
    threads : Vec<ThreadId>,
    enabled: bool,
    tree_arena: TreeArena
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
            threads: vec![],
            enabled: false,
            tree_arena: TreeArena::new()
        }
    }

    pub fn set_enable(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enable(&self) -> bool {
        self.enabled
    }

    pub fn on_thread_start(&mut self, thread: ThreadId) {
        //self.threads.push(thread);
    }

    pub fn on_thread_end(&mut self, thread: &ThreadId) {
        //self.threads.remove_item(thread);
//        if let Some(pos) = self.threads.iter().position(|x| *x == *thread) {
//            self.threads.remove(pos);
//        }
    }

    pub fn write_all_call_trees(&self, writer: &mut std::io::Write) {
        for (thread_id, call_tree) in self.tree_arena.get_all_call_trees() {
            let tree_name = &call_tree.get_top_node().data.name;
            writer.write_fmt(format_args!("Thread {}", tree_name));

            writer.write_all(call_tree.format_call_tree(true).as_bytes());
            writer.write_all("\n".as_bytes());
        }
    }

    pub fn add_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) {
        //merge to call stack tree
        for (i, stack_info) in stack_traces.iter().enumerate() {
            let mut cpu_time = -1i64;
            if let Ok(t) = jvm_env.get_thread_cpu_time(&stack_info.thread) {
                cpu_time = t as i64 / 1000;
            } else {
                warn!("get_thread_cpu_time error");
            }

            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
                let mut call_methods :Vec<(String, String)> = vec![];
                for stack_frame in &stack_info.frame_buffer {
                    let method_info = self.get_method_info(jvm_env, stack_frame.method);
                    call_methods.push((method_info.class.name.to_string(), method_info.method.name.to_string()));
                    //let call_name = format!("{}.{}()\n", &method_info.class.name, &method_info.method.name);
                }
                let call_tree = self.tree_arena.get_call_tree(&thread_info);
                call_tree.reset_top_call_stack_node();
                for (class_name, method_name) in &call_methods {
                    call_tree.begin_call(class_name, method_name)
                }
                call_tree.end_last_call(cpu_time);
                println!("add call stack: {} cpu_time:{}", thread_info.name, cpu_time);
            }else {
                warn!("Thread UNKNOWN [{:?}]: (cpu_time = {})", stack_info.thread, cpu_time);
            }
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
            } else {
                result.push_str(&format!("Thread UNKNOWN [{:?}]: (cpu_time = {}) \n", stack_info.thread, cpu_time));
            }

            for stack_frame in &stack_info.frame_buffer {
                let method_info = self.get_method_info(jvm_env, stack_frame.method);
                result.push_str(&format!("{}.{}()\n", &method_info.class.name, &method_info.method.name));
            }
        }
        result
    }

    fn get_method_info(&mut self, jvm_env: &Box<Environment>, method: JavaMethod) -> &MethodInfo {
        let method_id = MethodId { native_id: method };
        {
            self.method_cache.entry(method_id).or_insert_with(|| {
                let method = jvm_env.get_method_name(&method_id).unwrap();
                let class_id = jvm_env.get_method_declaring_class(&method_id).unwrap();
                let class = jvm_env.get_class_signature(&class_id).unwrap();
                MethodInfo {
                    method_id: method_id,
                    method,
                    class
                }
            });
        }

        self.method_cache.get(&method_id).unwrap()

    }
}

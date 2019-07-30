
use super::super::environment::jvmti::*;
use method::MethodId;

pub fn print_stack_traces(jvmti: &JVMTI, stack_traces: &Vec<JavaStackTrace>) {

    for (i, stack_info) in stack_traces.iter().enumerate() {
        println!("stack_info: {}, thread: {:?}, state: {:?}", (i+1), stack_info.thread, stack_info.state);

        let mut cpu_time = -1;
        match jvmti.get_thread_cpu_time(stack_info.thread) {
            Ok(t) => { cpu_time = t },
            Err(err) => {
                debug!("get_thread_cpu_time error: {:?}", err)
            }
        }

        if let Ok(thread_info) = jvmti.get_thread_info(&stack_info.thread) {
            println!("Thread [{:?}] {}: (state = {:?}, cpu_time = {}) ", stack_info.thread, thread_info.name, stack_info.state, cpu_time );
        }else {
            println!("Thread [{:?}] UNKNOWN: (state = UNKNOWN, cpu_time = {}) ", stack_info.thread, cpu_time);
        }

        for stack_frame in &stack_info.frame_buffer {
            let method_id = MethodId { native_id : stack_frame.method };
            let method = jvmti.get_method_name(&method_id).unwrap();
            let class_id = jvmti.get_method_declaring_class(&method_id).unwrap();
            let class = jvmti.get_class_signature(&class_id).unwrap();
            println!("{}.{}()", &class.name, &method.name);
        }
        println!("");
    }

}
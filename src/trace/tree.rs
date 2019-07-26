
use std::collections::HashMap;
use std::rc::*;
use std::borrow::Cow;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::Arc;
use time::Duration;

use thread::*;
use log::{debug, info, warn};


static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn get_next_nodeid() {
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
}



// thread safe
pub struct TreeArena {
    thread_trees: HashMap<ThreadId,ThreadData>,
    lock: RwLock<u32>
}

impl TreeArena {
    pub fn new() -> TreeArena {
        TreeArena {
            thread_trees: HashMap::new(),
            lock: RwLock::new(0)
        }
    }

//    pub fn get_call_tree(&self, thread: &Thread) -> Option<Arc<ThreadData>> {
//        self.thread_trees.write().unwrap().get_mut(&thread.id).map(|v| Arc::clone(v))
//    }

//    pub fn create_call_tree(&self, thread: &Thread) {
//        self.thread_trees.write().unwrap()
//            .insert(thread.id.clone(), Arc::new(ThreadData {
//                nodes: vec![TreeNode::newRootNode(&thread.name)],
//                root_node: NodeId{index: 0},
//                top_call_stack_node: NodeId{index: 0},
//            }));
//    }

    pub fn begin_call(&mut self, thread: &Thread, package: &String, class_name: &String, method_name: &String) {
        {
            let mut n = self.lock.write().unwrap();
            *n += 1;
            match self.thread_trees.get_mut(&thread.id) {
                Some(thread_data) => {
                    thread_data.begin_call(&package, &class_name, &method_name);
                },
                None => {
                    self.thread_trees.insert(thread.id.clone(), ThreadData {
                        nodes: vec![TreeNode::newRootNode(&thread.name)],
                        root_node: NodeId { index: 0 },
                        top_call_stack_node: NodeId { index: 0 },
                    });
                    let thread_data = self.thread_trees.get_mut(&thread.id).unwrap();
                    thread_data.begin_call(&package, &class_name, &method_name);
                    println!(" create call tree: [{:?}] [{}], total trees: {} ", thread.id.native_id, thread.name, self.thread_trees.len());
                    //println!(" create call tree failed: [{:?}] [{}] ", thread.id.native_id, thread.name);
                }
            }
        }
    }

    pub fn end_call(&mut self, thread: &Thread, package: &String, class_name: &String, method_name: &String, duration: &Duration) {
        let mut n = self.lock.write().unwrap();
        *n += 1;
        match self.thread_trees.get_mut(&thread.id) {
            Some(thread_data) => {
                thread_data.end_call(&package, &class_name, &method_name, &duration);
            },
            None => {}
        }
    }

    pub fn print_call_tree(&mut self, thread: &Thread) {
        match self.thread_trees.get(&thread.id) {
            Some(thread_data) => {
                println!("call tree of thread: [{:?}] [{}]", thread.id.native_id, thread.name);
                thread_data.print_call_tree();
            },
            None => {
                println!("call tree not found of thread: [{:?}] [{}]", thread.id.native_id, thread.name)
            }
        }
    }

    pub fn print_all(&self) {
        for (thread_id,thread_data) in self.thread_trees.iter() {
            println!("call tree of thread: [{:?}]", thread_id.native_id);
            thread_data.print_call_tree();
        }
    }

    pub fn clear(&mut self) {
        self.thread_trees.clear();
        println!("clear trace data");
    }
}


pub struct ThreadData {
    nodes: Vec<TreeNode>,
    root_node: NodeId,
    top_call_stack_node: NodeId
}

impl ThreadData {

    pub fn begin_call(&mut self, package: &String, class_name: &String, method_name: &String) {
        //let topNode = &mut self.nodes[self.top_call_stack_node.index];
        let call_name = TreeNode::get_node_key(package, class_name, method_name);

        //find exist call node
        let topNode = self.get_top_node();
        match topNode.find_child(&call_name) {
            Some(child_id) => {
                let node = self.get_node(child_id);
                self.top_call_stack_node = node.data.node_id.clone();
            },
            None => {
                //add new call node
                // Get the next free index
                let next_index = self.nodes.len();

                let topNode = self.get_mut_top_node();
                let node_data = TreeNode::newCallNode(topNode, next_index, package, class_name, method_name);
                self.top_call_stack_node = node_data.data.node_id.clone();

                // Push the node into the arena
                self.nodes.push(node_data);
            }
        }

    }

    pub fn end_call(&mut self, package: &String, class_name: &String, method_name: &String, duration: &Duration) {
        let call_name = TreeNode::get_node_key(package, class_name, method_name);
        //let top_node = self.nodes[self.top_call_stack_node.index];
        let top_node = self.get_mut_top_node();
        if top_node.data.name == call_name {
            top_node.data.call_duration += -duration.num_microseconds().unwrap();
            top_node.data.call_count += 1;

            debug!("end_call: {} {}, call_count:{}", call_name, duration, top_node.data.call_count);

            //pop stack
            //let parentNode = self.get_node(top_node.parent);
            //self.top_call_stack_node = top_node.parent.unwrap().clone();
            match &top_node.parent {
                Some(nodeid) => {
                    self.top_call_stack_node = nodeid.clone();
                },
                None => {
                    println!("parent node not found, pop call stack failed, call_name: {}, stack: {}, depth: {}",
                             call_name, top_node.data.path, top_node.data.depth)
                }
            }
        } else {
            println!("call name mismatch, pop call stack failed, call_name: {}, top_node:{}, stack:{}, depth: {} ",
                     call_name, top_node.data.name, top_node.data.path, top_node.data.depth);
        }
    }

    pub fn print_call_tree(&self) {

        self.print_tree_node(&self.root_node);
    }

    pub fn print_tree_node(&self, nodeid: &NodeId) {
        let node = self.get_node(&nodeid);
        for x in 0..node.data.depth {
            print!("  ");
        }
        let mut call_duration = node.data.call_duration;
        //sum all children duration of root
        if nodeid.index == 0 {
            for child in node.children.values() {
                call_duration += self.get_node(&child).data.call_duration;
            }
        }else {

        }
        println!("{}[calls={}, duration={}]", node.data.name, node.data.call_count, call_duration);

        for child in node.children.values() {
            self.print_tree_node(&child);
        }
    }

    pub fn get_top_node(&self) -> &TreeNode {
        &self.nodes[self.top_call_stack_node.index]
    }

    pub fn get_mut_top_node(&mut self) -> &mut TreeNode {
        self.nodes.get_mut(self.top_call_stack_node.index).unwrap()
    }

    pub fn get_node(&self, node_id: &NodeId) -> &TreeNode {
        &self.nodes[node_id.index]
    }
}

#[derive(Clone)]
pub struct NodeData {
    node_id: NodeId,
    depth: u32,
    name: String,
    path: String,
    call_count: u32, // call count
    call_duration: i64, // call duration
    children_size: u32 //children size
}

#[derive(Clone)]
pub struct NodeId {
    index: usize,
}

#[derive( Clone)]
pub struct TreeNode {
    data: NodeData,
    parent: Option<NodeId>,
    children: HashMap<String, NodeId>
}

impl TreeNode {

    pub fn newRootNode(name: &String) -> TreeNode {
        TreeNode{
            data : NodeData {
                node_id: NodeId{index:0},
                depth: 0,
                name: name.to_string(),
                path: name.to_string(),
                call_count: 0,
                call_duration: 0,
                children_size: 0,
            },
            parent: None,
            children: HashMap::new()
        }
    }

    pub fn newCallNode(parentNode: &mut TreeNode, next_index: usize, package: &String, class_name: &String, method_name: &String ) -> TreeNode {
        let name = TreeNode::get_node_key(package, class_name, method_name);

        //call path
        let mut path = parentNode.data.path.to_string();
        path += ";";
        path += name.as_str();

        let node_id = NodeId{index:next_index};

        parentNode.children.insert(name.clone(), node_id.clone());
        parentNode.data.children_size += 1;

        TreeNode{
            data : NodeData {
                node_id: node_id,
                name: name.to_string(),
                path: path.to_string(),
                depth: parentNode.data.depth + 1,
                call_count: 0,
                call_duration: 0,
                children_size: 0,
            },
            parent: Some(parentNode.data.node_id.clone()),
            children: HashMap::new(),
        }

    }

    fn get_node_key(package: &String, class_name: &String, method_name: &String) -> String {
        let mut name = String::from("");
        if package.len() > 0 {
            name += package.as_str();
            name += ".";
        }
        name += class_name.as_str();
        name += ".";
        name += method_name.as_str();
        name
    }

    fn find_child(&self, call_name: &String) -> Option<&NodeId> {
        self.children.get(call_name)
    }

}

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
use native::JavaLong;
use std::collections::hash_map::IterMut;


static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn get_next_nodeid() {
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
}



// assume thread safe, get lock outside
pub struct TreeArena {
    thread_trees: HashMap<JavaLong, CallStackTree>,
//    lock: RwLock<u32>
}

impl TreeArena {
    pub fn new() -> TreeArena {
        TreeArena {
            thread_trees: HashMap::new(),
            //lock: RwLock::new(0)
        }
    }

    pub fn get_all_call_trees(&self) -> &HashMap<JavaLong, CallStackTree>{
        &self.thread_trees
    }

    pub fn get_call_tree(&mut self, thread: &Thread) -> &mut CallStackTree {
        self.thread_trees.entry(thread.thread_id).or_insert_with(||{ CallStackTree::new(thread.thread_id, &thread.name) });
        self.thread_trees.get_mut(&thread.thread_id).unwrap()
    }

    pub fn begin_call(&mut self, thread: &Thread, class_name: &String, method_name: &String) {
//        let mut n = self.lock.write().unwrap();
//        *n += 1;
        match self.thread_trees.get_mut(&thread.thread_id) {
            Some(thread_data) => {
                thread_data.begin_call(&class_name, &method_name);
            },
            None => {
                self.thread_trees.insert(thread.thread_id, CallStackTree::new(thread.thread_id, &thread.name));
                let thread_data = self.thread_trees.get_mut(&thread.thread_id).unwrap();
                thread_data.begin_call(&class_name, &method_name);
                println!(" create call tree: [{:?}] [{}], total trees: {} ", thread.thread_id, thread.name, self.thread_trees.len());
            }
        }
    }

    pub fn end_call(&mut self, thread: &Thread, class_name: &String, method_name: &String, duration: i64) {
//        let mut n = self.lock.write().unwrap();
//        *n += 1;
        match self.thread_trees.get_mut(&thread.thread_id) {
            Some(thread_data) => {
                thread_data.end_call(&class_name, &method_name, duration);
            },
            None => {}
        }
    }

    pub fn format_call_tree(&mut self, thread: &Thread, compact: bool) -> String {
        match self.thread_trees.get(&thread.thread_id) {
            Some(thread_data) => {
                println!("call tree of thread: [{}] [{}]", thread.thread_id, thread.name);
                thread_data.format_call_tree(compact)
            },
            None => {
                println!("call tree not found of thread: [{}] [{}]", thread.thread_id, thread.name);
                String::from("[call tree not found]")
            }
        }
    }

    pub fn print_all(&self) {
        for (thread_id,thread_data) in self.thread_trees.iter() {
            println!("call tree of thread: [{}]", thread_id);
            println!("{}", thread_data.format_call_tree(false));
        }
    }

    pub fn clear(&mut self) {
        self.thread_trees.clear();
        println!("clear trace data");
    }
}


pub struct CallStackTree {
    nodes: Vec<TreeNode>,
    root_node: NodeId,
    top_call_stack_node: NodeId,
    pub total_duration: i64,
    pub thread_id: JavaLong
}

impl CallStackTree {

    pub fn new(thread_id: JavaLong, thread_name: &str) -> CallStackTree {
        CallStackTree {
            nodes: vec![TreeNode::newRootNode(thread_name)],
            root_node: NodeId { index: 0 },
            top_call_stack_node: NodeId { index: 0 },
            total_duration: 0,
            thread_id: thread_id
        }
    }

    pub fn reset_top_call_stack_node(&mut self) {
        self.top_call_stack_node = self.root_node;
    }

    pub fn begin_call(&mut self, class_name: &String, method_name: &String) {
        //let topNode = &mut self.nodes[self.top_call_stack_node.index];
        let call_name = TreeNode::get_node_key(class_name, method_name);

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
                let node_data = TreeNode::newCallNode(topNode, next_index, class_name, method_name);
                self.top_call_stack_node = node_data.data.node_id.clone();

                // Push the node into the arena
                self.nodes.push(node_data);
            }
        }

    }

    pub fn end_call(&mut self, class_name: &String, method_name: &String, duration: i64) {
        let call_name = TreeNode::get_node_key(class_name, method_name);
        //let top_node = self.nodes[self.top_call_stack_node.index];
        let top_node = self.get_mut_top_node();
        if top_node.data.name == call_name {
            top_node.data.call_duration += duration;
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
                             call_name, top_node.data.name, top_node.data.depth)
                }
            }
        } else {
            println!("call name mismatch, pop call stack failed, call_name: {}, top_node:{}, stack:{}, depth: {} ",
                     call_name, top_node.data.name, top_node.data.name, top_node.data.depth);
        }
    }

    pub fn end_last_call(&mut self, total_duration: i64) {
        let last_duration = self.total_duration;
        let top_node = self.get_mut_top_node();
        //ignore first call duration
        if(last_duration > 0){
            top_node.data.call_duration += (total_duration - last_duration);
        }
        top_node.data.call_count += 1;
        self.total_duration = total_duration;
    }

    //
    // compact: bool 是否为紧凑模式，即树结点深度使用数字表示。如果为false，则树深度使用多个' '表示
    //
    pub fn format_call_tree(&self, compact: bool) -> String {
        let mut result  = String::new();
        self.format_tree_node(&mut result,&self.root_node, compact);
        result
    }

    pub fn format_tree_node(&self, result: &mut String, nodeid: &NodeId, compact: bool) {
        let node = self.get_node(&nodeid);
        if compact {
            result.push_str(&format!("{},", node.data.depth));
        } else {
            for x in 0..node.data.depth {
                result.push_str(&format!("  "));
            }
        }
        let mut call_duration = node.data.call_duration;
        //sum all children duration of root
        if nodeid.index == 0 {
            for child in node.children.values() {
                call_duration += self.get_node(&child).data.call_duration;
            }
        }else {

        }
        result.push_str(&format!("{}[calls={}, duration={}]\n", node.data.name, node.data.call_count, call_duration as f64/1000_000.0));

        for child in node.children.values() {
            self.format_tree_node(result,&child, compact);
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

    pub fn get_root_node(&self) -> &TreeNode {
        &self.nodes[self.root_node.index]
    }
}

#[derive(Clone)]
pub struct NodeData {
    pub node_id: NodeId,
    pub depth: u32, // move to TreeNode
    pub name: String,
//    path: String,
    pub call_count: u32, // call count
    pub call_duration: i64, // call duration
    pub children_size: u32 //children size
}

#[derive(Clone, Copy)]
pub struct NodeId {
    index: usize,
}

#[derive( Clone)]
pub struct TreeNode {
    pub data: NodeData,
    parent: Option<NodeId>,
    children: HashMap<String, NodeId>
}

impl TreeNode {

    pub fn newRootNode(name: &str) -> TreeNode {
        TreeNode{
            data : NodeData {
                node_id: NodeId{index:0},
                depth: 0,
                name: name.to_string(),
//                path: name.to_string(),
                call_count: 0,
                call_duration: 0,
                children_size: 0,
            },
            parent: None,
            children: HashMap::new()
        }
    }

    pub fn newCallNode(parentNode: &mut TreeNode, next_index: usize, class_name: &String, method_name: &String ) -> TreeNode {
        let name = TreeNode::get_node_key(class_name, method_name);

        //call path
//        let mut path = parentNode.data.path.to_string();
//        path += ";";
//        path += name.as_str();

        let node_id = NodeId{index:next_index};

        parentNode.children.insert(name.clone(), node_id.clone());
        parentNode.data.children_size += 1;

        TreeNode{
            data : NodeData {
                node_id: node_id,
                name: name.to_string(),
//                path: path.to_string(),
                depth: parentNode.data.depth + 1,
                call_count: 0,
                call_duration: 0,
                children_size: 0,
            },
            parent: Some(parentNode.data.node_id.clone()),
            children: HashMap::new(),
        }

    }

    fn get_node_key(class_name: &String, method_name: &String) -> String {
        format!("{}.{}", class_name, method_name)
    }

    fn find_child(&self, call_name: &String) -> Option<&NodeId> {
        self.children.get(call_name)
    }

}
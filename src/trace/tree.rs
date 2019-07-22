
use std::collections::HashMap;
use std::rc::*;
use std::borrow::Cow;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::Arc;

use thread::*;

lazy_static! {
    static ref TREE_ARENA: TreeArena = TreeArena::new();
}

static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn get_next_nodeid() {
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
}


fn get_tree_arena() -> &'static TreeArena {
    &TREE_ARENA
}
// thread safe
pub struct TreeArena {
    thread_trees: RwLock<HashMap<ThreadId,Arc<ThreadData>>>
}

impl TreeArena {
    pub fn new() -> TreeArena {
        TreeArena {
            thread_trees: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_call_tree(&self, thread: &Thread) -> Option<Arc<ThreadData>> {
        self.thread_trees.write().unwrap().get_mut(&thread.id).map(|v| Arc::clone(v))
    }

    pub fn create_call_tree(&self, thread: &Thread) {
        self.thread_trees.write().unwrap()
            .insert(thread.id.clone(), Arc::new(ThreadData {
                nodes: vec![TreeNode::newRootNode(&thread.name)],
                root_node: NodeId{index: 0},
                top_call_stack_node: NodeId{index: 0},
            }));
    }
}


pub struct ThreadData {
    nodes: Vec<TreeNode>,
    root_node: NodeId,
    top_call_stack_node: NodeId
}

impl ThreadData {

    pub fn new_node(&mut self, package: &String, class_name: &String, method_name: &String) -> NodeId {
        // Get the next free index
        let next_index = self.nodes.len();
        let topNode = &mut self.nodes[self.top_call_stack_node.index];

        let node_data = TreeNode::newCallNode(topNode, next_index, package, class_name, method_name);
        // Push the node into the arena

        // Return the node identifier
        //NodeId { index: next_index }
        node_data.data.node_id
    }

}

#[derive(Clone)]
pub struct NodeData {
    node_id: NodeId,
    depth: u32,
    name: String,
    path: String,
    call_count: u32, // call count
    call_duration: u32, // call duration
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

        TreeNode{
            data : NodeData {
                node_id: node_id,
                name: name.to_string(),
                path: path.to_string(),
                depth:0,
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

}
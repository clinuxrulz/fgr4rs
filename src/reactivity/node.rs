use super::context::{batch, clear_dirty_flag_at_end, mark_nodes_to_be_updated, outer_node};
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

pub struct Node {
    pub update_op: RefCell<Option<Box<dyn FnMut()>>>,
    pub visited: Cell<bool>,
    pub dirty: Cell<bool>,
    pub dependencies: RefCell<Vec<Rc<dyn HasNode>>>,
    pub dependents: RefCell<Vec<Weak<dyn HasNode>>>,
}

pub trait HasNode {
    fn node(&self) -> &Node;
}

impl Node {
    pub fn new() -> Node {
        Node {
            update_op: RefCell::new(None),
            visited: Cell::new(false),
            dirty: Cell::new(false),
            dependencies: RefCell::new(Vec::new()),
            dependents: RefCell::new(Vec::new()),
        }
    }
}

pub struct NodeWithValue<A> {
    pub value: RefCell<Option<A>>,
    pub node: Node,
}

impl<A> NodeWithValue<A> {
    pub fn new(value: A) -> NodeWithValue<A> {
        NodeWithValue {
            value: RefCell::new(Some(value)),
            node: Node::new(),
        }
    }

    pub fn new2(node: Node) -> NodeWithValue<A> {
        NodeWithValue {
            value: RefCell::new(None),
            node,
        }
    }
}

impl<A> HasNode for NodeWithValue<A> {
    fn node(&self) -> &Node {
        &self.node
    }
}

pub struct NodeValRef<'a,A> {
    x: Ref<'a,Option<A>>,
    read: Cell<bool>,
    node: Rc<dyn HasNode>,
}

impl<'a,A> NodeValRef<'a,A> {
    pub fn new(node_with_value: &'a NodeWithValue<A>, node: Rc<dyn HasNode>) -> NodeValRef<'a,A> {
        NodeValRef {
            x: node_with_value.value.borrow(),
            read: Cell::new(false),
            node
        }
    }
}

impl<'a,A> Deref for NodeValRef<'a,A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        self.x.deref().as_ref().unwrap()
    }
}

impl<'a,A> Drop for NodeValRef<'a,A> {
    fn drop(&mut self) {
        if self.read.get() {
            if let Some(outer_node) = outer_node() {
                outer_node.node().dependencies.borrow_mut().push(self.node.clone());
                self.node.node().dependents.borrow_mut().push(Rc::downgrade(&outer_node));
            }
        }
    }
}

pub struct NodeValRefMut<'a,A> {
    x: Option<RefMut<'a,Option<A>>>,
    written: Cell<bool>,
    node: Rc<dyn HasNode>,
}

impl<'a,A> NodeValRefMut<'a,A> {
    pub fn new(node_with_value: &'a NodeWithValue<A>, node: Rc<dyn HasNode>) -> NodeValRefMut<'a,A> {
        NodeValRefMut {
            x: Some(node_with_value.value.borrow_mut()),
            written: Cell::new(false),
            node
        }
    }
}

impl<'a,A> Deref for NodeValRefMut<'a,A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        self.x.as_ref().unwrap().deref().as_ref().unwrap()
    }
}

impl<'a,A> DerefMut for NodeValRefMut<'a,A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written.set(true);
        self.x.as_mut().unwrap().deref_mut().as_mut().unwrap()
    }
}

impl<'a,A> Drop for NodeValRefMut<'a,A> {
    fn drop(&mut self) {
        self.x = None;
        if self.written.get() {
            batch(|| {
                self.node.node().dirty.set(true);
                clear_dirty_flag_at_end(Rc::downgrade(&self.node));
                mark_nodes_to_be_updated([Rc::downgrade(&self.node)].iter().cloned());
            });
        }
    }
}

use super::context::in_node;
use super::node::{HasNode, Node, NodeWithValue, NodeValRef};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub struct Memo<A> {
    data: Rc<NodeWithValue<A>>,
}

impl<A> Clone for Memo<A> {
    fn clone(&self) -> Self {
        Memo {
            data: self.data.clone(),
        }
    }
}

impl<A:'static> Memo<A> {
    pub fn new<K:FnMut()->A+'static>(mut k: K) -> Memo<A> {
        let node = Node::new();
        let forward_ref: Rc<RefCell<Option<Weak<NodeWithValue<A>>>>> = Rc::new(RefCell::new(None));
        let mut update;
        {
            let forward_ref = forward_ref.clone();
            update = move || {
                let node = forward_ref.borrow().as_ref().unwrap().upgrade().unwrap();
                let weak_node = Rc::downgrade(&node) as Weak<dyn HasNode>;
                for dependency in &*node.node.dependencies.borrow() {
                    dependency.node().dependents.borrow_mut().retain(|x| !x.ptr_eq(&weak_node));
                }
                node.node.dependencies.borrow_mut().clear();
                let r = in_node(
                    node,
                    || {
                        k()
                    }
                );
                r
            };
        }
        let r = Memo {
            data: Rc::new(NodeWithValue::new2(node)),
        };
        *(*forward_ref).borrow_mut() = Some(Rc::downgrade(&r.data));
        *r.data.value.borrow_mut() = Some(update());
        let node_with_value = r.data.clone();
        *r.data.node.update_op.borrow_mut() = Some(Box::new(move || {
            let r = update();
            *node_with_value.value.borrow_mut() = Some(r);
        }));
        r
    }

    pub fn read<'a>(&'a self) -> impl Deref<Target=A> + 'a {
        NodeValRef::new(&*self.data, self.data.clone())
    }
}
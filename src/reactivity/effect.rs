use super::batch;
use super::context::{in_node, push_effect};
use super::node::{HasNode, Node};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Effect {
    data: Rc<Node>,
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect {
            data: self.data.clone(),
        }
    }
}

impl Effect {
    pub fn new<K: FnMut() + 'static>(mut k: K) -> Effect {
        batch(|| {
            let this = Rc::new(RefCell::new(None));
            let node = Rc::new(Node::new(this.clone()));
            let this2 = Rc::downgrade(&node);
            *this.borrow_mut() = Some(this2.clone());
            let this3 = this2.clone();
            let k2 = Rc::new(RefCell::new(Box::new(move || {
                let node = this2.upgrade().unwrap();
                let weak_node = Rc::downgrade(&node) as Weak<dyn HasNode>;
                for dependency in &*node.dependencies.borrow() {
                    dependency.node().dependents.borrow_mut().retain(|x| !x.ptr_eq(&weak_node));
                }
                node.dependencies.borrow_mut().clear();
                in_node(
                    node,
                    || {
                        k();
                    }
                );
            }) as Box<dyn FnMut()>));
            {
                let node = node.clone();
                let k2 = k2.clone();
                let k3 = Rc::new(RefCell::new(Box::new(move || {
                    let _ = &node;
                    k2.borrow_mut()();
                }) as Box<dyn FnMut()>));
                push_effect(k3);
            }
            *node.update_op.borrow_mut() = Some(Box::new(move || {
                let node = this3.upgrade().unwrap();
                let k2 = k2.clone();
                let k3 = Rc::new(RefCell::new(Box::new(move || {
                    let _ = &node;
                    k2.borrow_mut()();
                }) as Box<dyn FnMut()>));
                push_effect(k3);
                false
            }));
            Effect {
                data: node,
            }
        })
    }

    pub fn merge(effects: Vec<Effect>) -> Effect {
        Effect::new(move || {
            let _ = &effects;
        })
    }
}

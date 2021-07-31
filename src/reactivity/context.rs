use super::node::HasNode;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct Context {
    pub transaction_depth: Cell<usize>,
    pub in_node: RefCell<Option<Rc<dyn HasNode>>>,
    pub to_be_updated: RefCell<Vec<Weak<dyn HasNode>>>,
    pub reset_visited_flags: RefCell<Vec<Weak<dyn HasNode>>>,
    pub clear_dirty_flags: RefCell<Vec<Weak<dyn HasNode>>>,
    pub effects: RefCell<Vec<Rc<RefCell<Box<dyn FnMut()>>>>>,
}

thread_local!(static CTX: Context =
    Context {
        transaction_depth: Cell::new(0),
        in_node: RefCell::new(None),
        to_be_updated: RefCell::new(Vec::new()),
        reset_visited_flags: RefCell::new(Vec::new()),
        clear_dirty_flags: RefCell::new(Vec::new()),
        effects: RefCell::new(Vec::new()),
    }
);

pub fn in_node<R,K:FnOnce()->R>(node: Rc<dyn HasNode>, k: K) -> R {
    let outer = CTX.with(|ctx| {
        let mut in_node = ctx.in_node.borrow_mut();
        let mut outer = Some(node);
        std::mem::swap(&mut *in_node, &mut outer);
        outer
    });
    let r = k();
    CTX.with(|ctx| {
        *ctx.in_node.borrow_mut() = outer;
    });
    r
}

pub fn outer_node() -> Option<Rc<dyn HasNode + 'static>> {
    CTX.with(|ctx|
        ctx.in_node.borrow().clone()
    )
}

pub fn mark_nodes_to_be_updated<I: Iterator<Item=Weak<dyn HasNode>>>(nodes: I) {
    CTX.with(|ctx| {
        let mut to_be_updated = ctx.to_be_updated.borrow_mut();
        let to_be_updated = &mut *to_be_updated;
        nodes.for_each(|node| to_be_updated.push(node));
    });
}

pub fn batch<R,K:FnOnce()->R>(k: K) -> R {
    CTX.with(|ctx| {
        ctx.transaction_depth.set(ctx.transaction_depth.get() + 1);
        let r = k();
        ctx.transaction_depth.set(ctx.transaction_depth.get() - 1);
        if ctx.transaction_depth.get() == 0 {
            propergate();
        }
        r
    })
}

pub fn propergate() {
    let mut effects = Vec::new();
    CTX.with(|ctx| ctx.transaction_depth.set(ctx.transaction_depth.get() + 1));
    loop {
        while let Some(weak_node) = CTX.with(|ctx| ctx.to_be_updated.borrow_mut().pop()) {
            if let Some(node) = weak_node.upgrade() {
                update_node(node);
            }
        }
        CTX.with(|ctx| {
            {
                let mut reset_visited_flags = ctx.reset_visited_flags.borrow_mut();
                for weak_node in &*reset_visited_flags {
                    if let Some(node) = weak_node.upgrade() {
                        node.node().visited.set(false);
                    }
                }
                reset_visited_flags.clear();
            }
            {
                let mut clear_dirty_flags = ctx.clear_dirty_flags.borrow_mut();
                for weak_node in &*clear_dirty_flags {
                    if let Some(node) = weak_node.upgrade() {
                        node.node().dirty.set(false);
                    }
                }
                clear_dirty_flags.clear();
            }
            std::mem::swap(&mut effects, &mut ctx.effects.borrow_mut());
        });
        if effects.is_empty() {
            break;
        }
        for effect in effects.drain(0..) {
            effect.borrow_mut()();
        }
    }
    CTX.with(|ctx| ctx.transaction_depth.set(ctx.transaction_depth.get() - 1));
}

pub fn update_node(node: Rc<dyn HasNode>) {
    if node.node().visited.get() {
        return;
    }
    let mut all_children_done = true;
    for dependency in &*node.node().dependencies.borrow() {
        if !dependency.node().visited.get() {
            all_children_done = false;
            CTX.with(|ctx| ctx.to_be_updated.borrow_mut().push(Rc::downgrade(dependency)));
        }
    }
    if !all_children_done {
        CTX.with(|ctx| ctx.to_be_updated.borrow_mut().insert(0, Rc::downgrade(&node)));
        return;
    }
    let any_dirty_children = node.node().dependencies.borrow().iter().any(|child| child.node().dirty.get());
    if any_dirty_children {
        if let Some(ref mut update) = &mut *node.node().update_op.borrow_mut() {
            let changed = update();
            if changed {
                node.node().dirty.set(true);
                clear_dirty_flag_at_end(Rc::downgrade(&node));
            }
        }
    }
    if node.node().dirty.get() {
        CTX.with(|ctx| {
            for dependent in &*node.node().dependents.borrow() {
                ctx.to_be_updated.borrow_mut().push(dependent.clone());
            }
        });
    }
    node.node().visited.set(true);
    reset_visited_flag_at_end(Rc::downgrade(&node));
}

pub fn reset_visited_flag_at_end(node: Weak<dyn HasNode>) {
    CTX.with(|ctx| ctx.reset_visited_flags.borrow_mut().push(node));
}

pub fn clear_dirty_flag_at_end(node: Weak<dyn HasNode>) {
    CTX.with(|ctx| ctx.clear_dirty_flags.borrow_mut().push(node));
}

pub fn push_effect(effect: Rc<RefCell<Box<dyn FnMut()>>>) {
    CTX.with(|ctx| ctx.effects.borrow_mut().push(effect));
}


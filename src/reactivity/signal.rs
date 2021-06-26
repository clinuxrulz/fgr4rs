use super::node::{HasNode, NodeWithValue, NodeValRef, NodeValRefMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};

pub struct Signal<A> {
    data: Rc<NodeWithValue<A>>,
}

impl<A> Clone for Signal<A> {
    fn clone(&self) -> Self {
        Signal {
            data: self.data.clone(),
        }
    }
}

impl<A:'static> Signal<A> {
    pub fn new(init_value: A) -> Signal<A> {
        let this: Rc<RefCell<Option<Weak<dyn HasNode>>>> = Rc::new(RefCell::new(None));
        let r = Signal {
            data: Rc::new(NodeWithValue::new(this.clone(), init_value))
        };
        *(*this).borrow_mut() = Some(Rc::downgrade(&r.data) as Weak<dyn HasNode>);
        r
    }

    pub fn read<'a>(&'a self) -> impl Deref<Target=A> + 'a {
        NodeValRef::new(&*self.data, self.data.clone())
    }

    pub fn write<'a>(&'a mut self) -> impl DerefMut<Target=A> + 'a {
        NodeValRefMut::new(&*self.data, self.data.clone())
    }
}

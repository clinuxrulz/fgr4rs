use super::node::{NodeWithValue, NodeValRef, NodeValRefMut};
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

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
        Signal {
            data: Rc::new(NodeWithValue::new(init_value))
        }
    }

    pub fn read<'a>(&'a self) -> impl Deref<Target=A> + 'a {
        NodeValRef::new(&*self.data, self.data.clone())
    }

    pub fn write<'a>(&'a mut self) -> impl DerefMut<Target=A> + 'a {
        NodeValRefMut::new(&*self.data, self.data.clone())
    }
}

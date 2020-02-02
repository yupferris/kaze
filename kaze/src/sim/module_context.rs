use crate::graph;

use typed_arena::Arena;

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ptr;

pub struct ModuleContext<'graph, 'arena> {
    pub instance_and_parent: Option<(
        &'graph graph::Instance<'graph>,
        &'arena ModuleContext<'graph, 'arena>,
    )>,
    children:
        RefCell<HashMap<*const graph::Instance<'graph>, &'arena ModuleContext<'graph, 'arena>>>,
}

impl<'graph, 'arena> ModuleContext<'graph, 'arena> {
    pub fn new() -> ModuleContext<'graph, 'arena> {
        ModuleContext {
            instance_and_parent: None,
            children: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_child(
        &'arena self,
        instance: &'graph graph::Instance<'graph>,
        arena: &'arena Arena<ModuleContext<'graph, 'arena>>,
    ) -> &'arena ModuleContext<'graph, 'arena> {
        let key = instance as *const _;
        if !self.children.borrow().contains_key(&key) {
            let child = arena.alloc(ModuleContext {
                instance_and_parent: Some((instance, self)),
                children: RefCell::new(HashMap::new()),
            });
            self.children.borrow_mut().insert(instance, child);
        }
        self.children.borrow()[&key]
    }
}

impl<'graph, 'arena> Eq for &'arena ModuleContext<'graph, 'arena> {}

impl<'graph, 'arena> Hash for &'arena ModuleContext<'graph, 'arena> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self as *const _ as usize)
    }
}

impl<'graph, 'arena> PartialEq for &'arena ModuleContext<'graph, 'arena> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(*self, *other)
    }
}

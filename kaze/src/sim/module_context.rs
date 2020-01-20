use crate::graph;

use typed_arena::Arena;

use std::cell::RefCell;
use std::collections::HashMap;

pub struct ModuleContext<'graph, 'arena> {
    pub instance_and_parent: Option<(
        &'graph graph::Instance<'graph>,
        &'arena ModuleContext<'graph, 'arena>,
    )>,
    pub children:
        RefCell<HashMap<*const graph::Instance<'graph>, &'arena ModuleContext<'graph, 'arena>>>,
}

impl<'graph, 'arena> ModuleContext<'graph, 'arena> {
    pub fn new(
        instance_and_parent: Option<(
            &'graph graph::Instance<'graph>,
            &'arena ModuleContext<'graph, 'arena>,
        )>,
    ) -> ModuleContext<'graph, 'arena> {
        ModuleContext {
            instance_and_parent,
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
            let child = arena.alloc(ModuleContext::new(Some((instance, self))));
            self.children.borrow_mut().insert(instance, child);
        }
        self.children.borrow()[&key]
    }
}

use std::rc::Rc;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Stack<T: Clone + Eq + PartialEq> {
    head: Option<Rc<StackNode<T>>>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct StackNode<T: Clone + Eq + PartialEq> {
    datum: T,
    next: Option<Rc<StackNode<T>>>,
}

impl<T: Clone + Eq + PartialEq> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { head: None }
    }

    pub fn push(&self, datum: T) -> Stack<T> {
        Stack {
            head: Some(Rc::new(StackNode {
                datum,
                next: self.head.clone(),
            })),
        }
    }

    pub fn pop(&self) -> Option<(T, Stack<T>)> {
        self.head.as_ref().map(|node| {
            (
                node.datum.clone(),
                Stack {
                    head: node.next.clone(),
                },
            )
        })
    }
}

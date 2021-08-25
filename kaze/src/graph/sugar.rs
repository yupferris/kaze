use super::signal::*;

/// **UNSTABLE:** Provides a convenient way to write conditional combinational logic.
///
/// # Panics
///
/// Since this construct wraps the returned values with [`Signal::mux`], any panic conditions from that method apply to the generated code as well.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// let m = c.module("MyModule");
/// let i = m.input("i", 1);
/// let invert = m.input("invert", 1);
/// let o = if_(invert, {
///     !i
/// }).else_({
///     i
/// });
/// m.output("o", o);
/// ```
// TODO: Can we constrain T more than this to make sure it's only a supported type?
pub fn if_<'a, T>(cond: &'a Signal<'a>, when_true: T) -> If<'a, T> {
    If::new(cond, when_true)
}

#[doc(hidden)]
pub struct If<'a, T> {
    cond: &'a Signal<'a>,
    when_true: T,
}

impl<'a, T> If<'a, T> {
    fn new(cond: &'a Signal<'a>, when_true: T) -> If<'a, T> {
        If { cond, when_true }
    }

    pub fn else_if(self, cond: &'a Signal<'a>, when_true: T) -> ElseIf<'a, T> {
        ElseIf {
            parent: ElseIfParent::If(self),
            cond,
            when_true,
        }
    }
}

impl<'a> If<'a, &'a Signal<'a>> {
    pub fn else_(self, when_false: &'a Signal<'a>) -> &Signal<'a> {
        self.cond.mux(self.when_true, when_false)
    }
}

macro_rules! replace_tt { ($t:tt, $($i:tt)*) => { $($i)* } }

macro_rules! generate_if {
    (($($number: tt),*)) => {
        impl<'a> If<'a,(
            $(&'a replace_tt!($number, Signal<'a>)),*,
        )> {
            pub fn else_(self, when_false: (
                $(&'a replace_tt!($number, Signal<'a>)),*,
            )) -> ($(&'a replace_tt!($number, Signal<'a>)),*,) {
                (
                    $(
                        self.cond.mux(self.when_true.$number, when_false.$number)
                    ),*,
                )
            }
        }
    };
}
generate_if!((0));
generate_if!((0, 1));
generate_if!((0, 1, 2));
generate_if!((0, 1, 2, 3));
generate_if!((0, 1, 2, 3, 4));
generate_if!((0, 1, 2, 3, 4, 5));
generate_if!((0, 1, 2, 3, 4, 5, 6));
generate_if!((0, 1, 2, 3, 4, 5, 6, 7));
generate_if!((0, 1, 2, 3, 4, 5, 6, 7, 8));
generate_if!((0, 1, 2, 3, 4, 5, 6, 7, 8, 9));
generate_if!((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10));
generate_if!((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11));

enum ElseIfParent<'a, T> {
    If(If<'a, T>),
    ElseIf(Box<ElseIf<'a, T>>),
}

#[doc(hidden)]
pub struct ElseIf<'a, T> {
    parent: ElseIfParent<'a, T>,
    cond: &'a Signal<'a>,
    when_true: T,
}

impl<'a, T> ElseIf<'a, T> {
    pub fn else_if(self, cond: &'a Signal<'a>, when_true: T) -> ElseIf<'a, T> {
        ElseIf {
            parent: ElseIfParent::ElseIf(Box::new(self)),
            cond,
            when_true,
        }
    }
}

impl<'a> ElseIf<'a, &'a Signal<'a>> {
    pub fn else_(self, when_false: &'a Signal<'a>) -> &Signal<'a> {
        let ret = self.cond.mux(self.when_true, when_false);
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

macro_rules! generate_else_if {
    (($($number: tt),*)) => {
        impl<'a> ElseIf<'a, ($(&'a replace_tt!($number, Signal<'a>)),*,)> {
            pub fn else_(self, when_false: (
                $(&'a replace_tt!($number, Signal<'a>)),*,
            )) -> ($(&'a replace_tt!($number, Signal<'a>)),*,) {
                let ret = ($(
                    self.cond.mux(self.when_true.$number, when_false.$number)
                ),*,);

                match self.parent {
                    ElseIfParent::If(parent) => parent.else_(ret),
                    ElseIfParent::ElseIf(parent) => parent.else_(ret),
                }
            }
        }
    };
}

generate_else_if!((0));
generate_else_if!((0, 1));
generate_else_if!((0, 1, 2));
generate_else_if!((0, 1, 2, 3));
generate_else_if!((0, 1, 2, 3, 4));
generate_else_if!((0, 1, 2, 3, 4, 5));
generate_else_if!((0, 1, 2, 3, 4, 5, 6));
generate_else_if!((0, 1, 2, 3, 4, 5, 6, 7));
generate_else_if!((0, 1, 2, 3, 4, 5, 6, 7, 8));
generate_else_if!((0, 1, 2, 3, 4, 5, 6, 7, 8, 9));
generate_else_if!((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10));
generate_else_if!((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11));
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
/// let p = Context::new();
///
/// let m = p.module("m", "MyModule");
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
pub fn if_<'a, T>(cond: &'a dyn Signal<'a>, when_true: T) -> If<'a, T> {
    If::new(cond, when_true)
}

#[doc(hidden)]
pub struct If<'a, T> {
    cond: &'a dyn Signal<'a>,
    when_true: T,
}

impl<'a, T> If<'a, T> {
    fn new(cond: &'a dyn Signal<'a>, when_true: T) -> If<'a, T> {
        If { cond, when_true }
    }

    pub fn else_if(self, cond: &'a dyn Signal<'a>, when_true: T) -> ElseIf<'a, T> {
        ElseIf {
            parent: ElseIfParent::If(self),
            cond,
            when_true,
        }
    }
}

impl<'a, T: Into<&'a dyn Signal<'a>>> If<'a, T> {
    pub fn else_<F: Into<&'a dyn Signal<'a>>>(self, when_false: F) -> &'a dyn Signal<'a> {
        self.cond.mux(self.when_true.into(), when_false.into())
    }
}

macro_rules! replace_tt { ($t:tt, $($i:tt)*) => { $($i)* } }

macro_rules! generate_if {
    (($($number: tt, $t: tt, $f: tt),*)) => {
        impl<'a, $($t: Into<&'a dyn Signal<'a>>),*,> If<'a, ($($t),*,)> {
            pub fn else_<$($f: Into<&'a dyn Signal<'a>>),*,>(self, when_false: ($($f),*,)) -> ($(&'a replace_tt!($number, dyn Signal<'a>)),*,) {
                (
                    $(self.cond.mux(self.when_true.$number.into(), when_false.$number.into())),*,
                )
            }
        }
    };
}

generate_if!((0, T0, F0));
generate_if!((0, T0, F0, 1, T1, F1));
generate_if!((0, T0, F0, 1, T1, F1, 2, T2, F2));
generate_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3));
generate_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4));
generate_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5));
generate_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6));
generate_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7
));
generate_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8
));
generate_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8, 9, T9, F9
));
generate_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8, 9, T9, F9, 10, T10, F10
));
generate_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8, 9, T9, F9, 10, T10, F10, 11, T11, F11
));

enum ElseIfParent<'a, T> {
    If(If<'a, T>),
    ElseIf(Box<ElseIf<'a, T>>),
}

#[doc(hidden)]
pub struct ElseIf<'a, T> {
    parent: ElseIfParent<'a, T>,
    cond: &'a dyn Signal<'a>,
    when_true: T,
}

impl<'a, T> ElseIf<'a, T> {
    pub fn else_if(self, cond: &'a dyn Signal<'a>, when_true: T) -> ElseIf<'a, T> {
        ElseIf {
            parent: ElseIfParent::ElseIf(Box::new(self)),
            cond,
            when_true,
        }
    }
}

impl<'a, T: Into<&'a dyn Signal<'a>>> ElseIf<'a, T> {
    pub fn else_<F: Into<&'a dyn Signal<'a>>>(self, when_false: F) -> &'a dyn Signal<'a> {
        let ret = self.cond.mux(self.when_true.into(), when_false.into());
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

macro_rules! generate_else_if {
    (($($number: tt, $t: tt, $f: tt),*)) => {
        impl<'a, $($t: Into<&'a dyn Signal<'a>>),*,> ElseIf<'a, ($($t),*,)> {
            pub fn else_<$($f: Into<&'a dyn Signal<'a>>),*,>(self, when_false: ($($f),*,)) -> ($(&'a replace_tt!($number, dyn Signal<'a>)),*,) {
                let ret = (
                    $(self.cond.mux(self.when_true.$number.into(), when_false.$number.into())),*,
                );
                match self.parent {
                    ElseIfParent::If(parent) => parent.else_(ret),
                    ElseIfParent::ElseIf(parent) => parent.else_(ret),
                }
            }
        }
    };
}

generate_else_if!((0, T0, F0));
generate_else_if!((0, T0, F0, 1, T1, F1));
generate_else_if!((0, T0, F0, 1, T1, F1, 2, T2, F2));
generate_else_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3));
generate_else_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4));
generate_else_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5));
generate_else_if!((0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6));
generate_else_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7
));
generate_else_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8
));
generate_else_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8, 9, T9, F9
));
generate_else_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8, 9, T9, F9, 10, T10, F10
));
generate_else_if!((
    0, T0, F0, 1, T1, F1, 2, T2, F2, 3, T3, F3, 4, T4, F4, 5, T5, F5, 6, T6, F6, 7, T7, F7, 8, T8,
    F8, 9, T9, F9, 10, T10, F10, 11, T11, F11
));

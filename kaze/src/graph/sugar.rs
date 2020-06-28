use super::signal::*;

/// **UNSTABLE:** Provides a convenient way to write conditional combinational logic.
///
/// # Panics
///
/// Since this construct wraps the returned values with [`mux`], any panic conditions from that method apply to the generated code as well.
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
///
/// [`mux`]: ./struct.Signal.html#method.mux
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

// TODO: Come up with a nice way to generate these definitions with macros
impl<'a> If<'a, (&'a Signal<'a>,)> {
    pub fn else_(self, when_false: (&'a Signal<'a>,)) -> (&Signal<'a>,) {
        (self.cond.mux(self.when_true.0, when_false.0),)
    }
}

impl<'a> If<'a, (&'a Signal<'a>, &'a Signal<'a>)> {
    pub fn else_(self, when_false: (&'a Signal<'a>, &'a Signal<'a>)) -> (&Signal<'a>, &Signal<'a>) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
        )
    }
}

impl<'a> If<'a, (&'a Signal<'a>, &'a Signal<'a>, &'a Signal<'a>)> {
    pub fn else_(
        self,
        when_false: (&'a Signal<'a>, &'a Signal<'a>, &'a Signal<'a>),
    ) -> (&Signal<'a>, &Signal<'a>, &Signal<'a>) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (&Signal<'a>, &Signal<'a>, &Signal<'a>, &Signal<'a>) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
            self.cond.mux(self.when_true.9, when_false.9),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
            self.cond.mux(self.when_true.9, when_false.9),
            self.cond.mux(self.when_true.10, when_false.10),
        )
    }
}

impl<'a>
    If<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
            self.cond.mux(self.when_true.9, when_false.9),
            self.cond.mux(self.when_true.10, when_false.10),
            self.cond.mux(self.when_true.11, when_false.11),
        )
    }
}

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

// TODO: Come up with a nice way to generate these definitions with macros
impl<'a> ElseIf<'a, (&'a Signal<'a>,)> {
    pub fn else_(self, when_false: (&'a Signal<'a>,)) -> (&Signal<'a>,) {
        let ret = (self.cond.mux(self.when_true.0, when_false.0),);
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a> ElseIf<'a, (&'a Signal<'a>, &'a Signal<'a>)> {
    pub fn else_(self, when_false: (&'a Signal<'a>, &'a Signal<'a>)) -> (&Signal<'a>, &Signal<'a>) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a> ElseIf<'a, (&'a Signal<'a>, &'a Signal<'a>, &'a Signal<'a>)> {
    pub fn else_(
        self,
        when_false: (&'a Signal<'a>, &'a Signal<'a>, &'a Signal<'a>),
    ) -> (&Signal<'a>, &Signal<'a>, &Signal<'a>) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (&Signal<'a>, &Signal<'a>, &Signal<'a>, &Signal<'a>) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
            self.cond.mux(self.when_true.9, when_false.9),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
            self.cond.mux(self.when_true.9, when_false.9),
            self.cond.mux(self.when_true.10, when_false.10),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

impl<'a>
    ElseIf<
        'a,
        (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    >
{
    pub fn else_(
        self,
        when_false: (
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
            &'a Signal<'a>,
        ),
    ) -> (
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
        &Signal<'a>,
    ) {
        let ret = (
            self.cond.mux(self.when_true.0, when_false.0),
            self.cond.mux(self.when_true.1, when_false.1),
            self.cond.mux(self.when_true.2, when_false.2),
            self.cond.mux(self.when_true.3, when_false.3),
            self.cond.mux(self.when_true.4, when_false.4),
            self.cond.mux(self.when_true.5, when_false.5),
            self.cond.mux(self.when_true.6, when_false.6),
            self.cond.mux(self.when_true.7, when_false.7),
            self.cond.mux(self.when_true.8, when_false.8),
            self.cond.mux(self.when_true.9, when_false.9),
            self.cond.mux(self.when_true.10, when_false.10),
            self.cond.mux(self.when_true.11, when_false.11),
        );
        match self.parent {
            ElseIfParent::If(parent) => parent.else_(ret),
            ElseIfParent::ElseIf(parent) => parent.else_(ret),
        }
    }
}

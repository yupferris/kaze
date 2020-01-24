/// **UNSTABLE:** Provides a convenient way to write conditional combinational logic.
///
/// # Panics
///
/// Since this macro rewrites the referenced variable assignments using [`mux`], any panic conditions from that method apply to the generated code as well.
///
/// # Examples
///
/// ```
/// use kaze::*;
///
/// let c = Context::new();
///
/// let m = c.module("my_module");
/// let i = m.input("i", 1);
/// let invert = m.input("invert", 1);
/// let mut o = i;
/// kaze_sugar! {
///     if (invert) {
///         o = !o; // Equivalent to `o = invert.mux(!o, o);`
///     }
/// }
/// m.output("o", o);
/// ```
///
/// [`mux`]: ./struct.Signal.html#method.mux
#[macro_export(local_inner_macros)]
macro_rules! kaze_sugar {
    ($($contents:tt)*) => {
        kaze_sugar_impl!([], [ $($contents)* ])
    };
}

#[doc(hidden)]
#[macro_export]
// TODO: This formulation can generate a lot of extra mux's with the same or similar conditions, and should be revisited!
macro_rules! kaze_sugar_impl {
    // [selector], [token stream]

    // No selector cases
    ([], [ $name:ident = $value:expr; $($rest:tt)* ]) => {
        $name = $value;
        kaze_sugar_impl!([], [ $($rest)* ]);
    };
    ([], [ if ($sel:expr) { $($inner:tt)* } $($rest:tt)* ]) => {
        kaze_sugar_impl!([ $sel ], [ $($inner)* ]);
        kaze_sugar_impl!([], [ $($rest)* ]);
    };
    ([], []) => {};

    // Selector cases
    ([ $sel:expr ], [ $name:ident = $value:expr; $($rest:tt)* ]) => {
        let prev = $name;
        kaze_sugar_impl!([ $sel ], [ $($rest)* ]);
        $name = $sel.mux($value, prev)
    };
    ([ $prev_sel:expr ], [ if ($sel:expr) { $($inner:tt)* } $($rest:tt)* ]) => {
        kaze_sugar_impl!([ $prev_sel & $sel ], [ $($inner)* ]);
        kaze_sugar_impl!([ $prev_sel ], [ $($rest)* ]);
    };
    ([ $_:expr ], []) => {};
}

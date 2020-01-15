// TODO: Better name?
#[macro_export(local_inner_macros)]
macro_rules! kaze_sugar {
    ($($contents:tt)*) => {
        kaze_sugar_impl!([], [ $($contents)* ])
    };
}

#[doc(hidden)]
#[macro_export]
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

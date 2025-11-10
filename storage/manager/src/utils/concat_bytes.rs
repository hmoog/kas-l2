#[macro_export]
macro_rules! concat_bytes {
    ( $( $x:expr ),* $(,)? ) => {{
        let total_len = 0 $( + $x.len() )*;
        let mut out = ::std::vec::Vec::with_capacity(total_len);
        $(
            out.extend_from_slice($x);
        )*
        out
    }};
}

/// Generic macros for applying mask and inserting a value
#[macro_export]
macro_rules! mask_and_insert_value {
    ($orig:expr, $value:expr, $mask:expr, <<, $shift:expr, $T:ty) => {{
        let orig_cast: $T = $orig.into();
        let value_cast: $T = $value.into();
        let mask_cast: $T = $mask.into();
        ((orig_cast & !(mask_cast << $shift)) | ((value_cast & mask_cast) << $shift))
    }};
    ($orig:expr, $value:expr, $mask:expr, >>, $shift:expr, $T:ty) => {{
        let orig_cast: $T = $orig.into();
        let value_cast: $T = $value.into();
        let mask_cast: $T = $mask.into();
        ((orig_cast & !(mask_cast >> $shift)) | ((value_cast & mask_cast) >> $shift))
    }};
    ($orig:expr, $value:expr, $mask:expr, $T:ty) => {{
        let orig_cast: $T = $orig.into();
        let value_cast: $T = $value.into();
        let mask_cast: $T = $mask.into();
        ((orig_cast & !mask_cast) | (value_cast & mask_cast))
    }};
}

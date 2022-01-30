macro_rules! quick_impl {
    (From<$from:ty> for $for:ty) => {
        impl From<$from> for $for {
            fn from(err: $from) -> Self {
                Self::new(err)
            }
        }
    };

    (From<$from:ty> for $for:ty, $variant:expr) => {
        impl From<$from> for $for {
            fn from(err: $from) -> Self {
                $variant(From::from(err))
            }
        }
    };
}

pub(crate) use quick_impl;

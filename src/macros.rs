#[macro_export]
macro_rules! list {
    (
        $( $value:expr ),* $(,)*
    ) => {
        {
            let mut list = MalList::new();
            $(
                list.push_back($value.into());
            )*
            list
        }
    }
}



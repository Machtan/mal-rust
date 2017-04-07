#[macro_export]
macro_rules! list_with_sym {
    (
        $sym:expr $(, $value:expr )* $(,)*
    ) => {
        {
            let mut list = MalList::new();
            list.push_back(Symbol::new($sym).into());
            $(
                list.push_back($value.into());
            )*
            list
        }
    }
}

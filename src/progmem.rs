#[macro_export]
macro_rules! progmem {
    ($vis:vis static progmem $name:ident: $ty:ty = $value:expr;) => {
        #[link_section = ".progmem.data"]
        $vis static: $ty = $value;
        $vis static $name: ProgMem<$ty> = ProgMem

    }
}
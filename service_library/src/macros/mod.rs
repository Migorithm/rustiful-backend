#[macro_export]
macro_rules! get_for_test {
    ($name:ident, &$ret:ty) => {
        #[cfg(test)]
        pub fn $name(&self) -> &$ret {
            &self.$name
        }
    };
    ($name:ident, $ret:ty) => {
        #[cfg(test)]
        pub fn $name(&self) -> $ret {
            self.$name
        }
    };
}

#[macro_export]
macro_rules! set {
    ($name:ident, $func:ident,$type:ty) => {
        pub fn $func(&mut self, $name: $type) {
            self.0.$name = $name.into()
        }
    };
}

pub use get_for_test;
pub use set;

#[macro_use]
mod log;

pub mod x;
pub mod core;
pub mod layouts;
pub mod manager;

pub(crate) mod util;

pub use crate::core::types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

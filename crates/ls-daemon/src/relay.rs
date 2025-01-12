use std::net::ToSocketAddrs;

pub trait Relay {}

pub struct WebRelay;

impl WebRelay {
    pub fn connect<S: ToSocketAddrs>(addr: S) {}
}

#[cfg(test)]
mod unit_test {
    use super::*;

    struct TestRelay {}

    impl Relay for TestRelay {}

    #[test]
    pub fn hello() {
        let _ = TestRelay {};
        assert_eq!(1 + 1, 2);
    }
}

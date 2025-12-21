use core::convert::Infallible;

pub trait InfallibleResultExt {
    fn infallible(self);
}

impl<T> InfallibleResultExt for Result<T, Infallible> {
    fn infallible(self) {
        self.expect("Result is infallible.");
    }
}

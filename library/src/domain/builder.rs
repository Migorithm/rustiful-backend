pub trait Builder<T> {
    fn new() -> Self;
    fn build(self) -> T;
}

pub trait Buildable<Target, B: Builder<Target>> {
    fn builder() -> B;
}

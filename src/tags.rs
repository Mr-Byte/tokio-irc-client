pub trait Tag<'a> {
    fn name() -> &'static str;
    fn parse(tag: &'a str) -> Option<Self> where Self: Sized;
}
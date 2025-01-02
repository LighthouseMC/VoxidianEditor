use std::iter;


pub struct SkipLast<I : Iterator>(iter::Peekable<I>);
impl<I : Iterator> Iterator for SkipLast<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next();
        self.0.peek().and_then(|_| next)
    }
}


pub trait IteratorExt : Iterator + Sized {

    fn skip_last(self) -> SkipLast<Self> { SkipLast(self.peekable()) }

}
impl<I : Iterator> IteratorExt for I { }
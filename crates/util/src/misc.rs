pub trait VecExt<T> {
    fn sorted_insert(&mut self, item: T);
}

impl<T: Ord> VecExt<T> for Vec<T> {
    #[inline]
    fn sorted_insert(&mut self, element: T) {
        match self.binary_search(&element) {
            Ok(idx) => {
                self[idx] = element;
            }
            Err(idx) => {
                self.insert(idx, element);
            }
        }
    }
}

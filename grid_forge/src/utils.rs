// pub struct VecUniq<T: PartialEq> {
//     inner: Vec<T>,
// }

// impl<T: PartialEq> VecUniq<T> {
//     pub fn new() -> Self {
//         Self { inner: Vec::new() }
//     }

//     pub fn inner(&self) -> &Vec<T> {
//         &self.inner
//     }

//     pub fn inner_mut(&mut self) -> &mut Vec<T> {
//         &mut self.inner
//     }

//     pub fn push(&mut self, elem: T) {
//         if self.inner.contains(&elem) {
//             return;
//         }
//         self.inner.push(elem);
//     }
// }

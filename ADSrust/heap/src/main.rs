fn main() {
    use crate::heap::Heap;
    use std::rc::Rc;
}

mod heap {
    use std::{cmp::Ordering, usize};

    struct Children {
        left_child: usize,
        right_child: usize,
    }

    pub struct Heap<T>(Vec<T>, usize);

    impl<T: Ord> Heap<T> {
        pub fn new() -> Self {
            let vec: Vec<T> = Vec::new();
            Heap(vec, 0)
        }

        pub fn enqueue(mut self: Self, item: T) {
            self.0.push(item);
            self.1 += 1;
            let y = self.1;
            self.check_swap_parent(y);
        }

        fn findparent(index: usize) -> usize {
            (index - 1) / 2
        }

        fn find_children(index: usize) -> Children {
            Children {
                left_child: index * 2 + 1,
                right_child: index * 2 + 2,
            }
        }

        fn check_swap_parent(mut self: Self, index: usize) {
            if index == 0 {
                return;
            }
            let parent = Self::findparent(index);
            if self.0.get(index).unwrap() > self.0.get(parent).unwrap() {
                self.0.swap(index, parent);
                self.check_swap_parent(parent);
            }
        }

        fn check_swap_children(mut self: Self, children: Children) {
            let small_child = match self
                .0
                .get(children.left_child)
                .unwrap()
                .cmp(&self.0.get(children.right_child).unwrap())
            {
                Ordering::Less => children.left_child,
                _ => children.right_child,
            };
        }

        pub fn dequeue(mut self) -> T {
            let y = self.1;
            self.0.swap(0, y);
            let x = self.0.pop();
            self.check_swap_children(Self::find_children(0));
            x.unwrap()
        }
    }
}

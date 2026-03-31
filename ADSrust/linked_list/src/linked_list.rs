mod linked {
    use std::{ops::Deref, rc::Rc};

    pub struct Node<T> {
        next: Option<Rc<Node<T>>>,
        item: T,
    }

    pub struct LinkedList<T> {
        head: Option<Rc<Node<T>>>,
        tail: Option<Rc<Node<T>>>,
        size: u32,
    }

    impl<T> Node<T> {
        fn new(item: T, next: Option<Rc<Node<T>>>) -> Self {
            Node { next, item }
        }
    }

    impl<T> LinkedList<T> {
        pub fn new() -> Self {
            LinkedList {
                head: None,
                tail: None,
                size: 0,
            }
        }

        fn init_list(mut self: Self, item: T) {
            let x: Rc<Node<T>> = Rc::new(Node::new(item, None));
            self.head = Some(Rc::clone(&x));
            self.tail = Some(Rc::clone(&x));
            self.size += 1;
        }

        pub fn add_first(mut self: Self, item: T) {
            match self.head {
                Some(node) => {
                    self.head = Some(Rc::new(Node::new(item, Some(Rc::clone(&node)))));
                }
                None => self.init_list(item),
            }
        }

        pub fn add_last(mut self: Self, item: T) {
            match self.tail {
                Some(_node) => {
                    let temp = Rc::new(Node::new(item, None));
                    //    Rc::try_unwrap(self.tail.unwrap()).next = Some(Rc::clone(&temp));
                    self.tail = Some(Rc::clone(&temp));
                }
                None => self.init_list(item),
            }
        }

        /*pub fn get(self: Self, index: u32) -> T {
            if let None = self.head {
                panic!("Index out of bounds!");
            }
            //let mut x: Node<T> = Rc::try_unwrap(self.head.unwrap());
            for _i in 0..index {
                if let None = x.next {
                    panic!("Index out of bounds!");
                }
                //  x = Rc::try_unwrap(x.next.unwrap());
            }
            x.item
        }*/
    }
}

mod leafy {
    use std::ptr::NonNull;
    pub struct Node<T> {
        next: Option<NonNull<Node<T>>>,
        item: T,
    }

    pub struct LinkedList<T> {
        head: Option<Node<T>>,
        size: u32,
    }

    impl<T> Node<T> {
        fn new(item: T, next: Option<NonNull<Node<T>>>) -> Self {
            Node { next, item }
        }
    }

    impl<T> LinkedList<T> {
        pub fn new() -> Self {
            LinkedList {
                head: None,
                size: 0,
            }
        }

        fn init_list(mut self: Self, item: T) {
            let x: Node<T> = Node::new(item, None);
            self.head = Some(x);
            self.size += 1;
        }

        pub fn add_first(mut self: Self, item: T) {
            match self.head {
                Some(node) => {
                    self.head = Some(Node::new(item, Some(NonNull::new(&node))));
                }
                None => self.init_list(item),
            }
        }

        pub fn get(self: Self, index: u32) -> T {
            if let None = self.head {
                panic!("Index out of bounds!");
            }
            //let mut x: Node<T> = Rc::try_unwrap(self.head.unwrap());
            for _i in 0..index {
                if let None = x.next {
                    panic!("Index out of bounds!");
                }
                //  x = Rc::try_unwrap(x.next.unwrap());
            }
            x.item
        }
    }
}

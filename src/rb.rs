pub struct RingBuffer<T, const CAP: usize> {
    rb: [T; CAP],
    head: usize,
    tail: usize,
}

#[derive(Debug)]
pub enum RbError {
    NoMoreSpace,
    IsEmpty,
}

#[allow(dead_code)]
impl<T, const CAP: usize> RingBuffer<T, CAP>
where
    T: Default + Copy,
{
    pub fn new() -> Self {
        RingBuffer {
            rb: [Default::default(); CAP],
            head: 0,
            tail: 0,
        }
    }

    pub fn put(&mut self, elem: T) -> Result<(), RbError> {
        if (self.head + 1) % CAP == self.tail {
            Err(RbError::NoMoreSpace)
        } else {
            self.rb[self.head] = elem;
            self.head = (self.head + 1) % CAP;
            Ok(())
        }
    }

    pub fn get(&mut self) -> Result<T, RbError> {
        if self.head == self.tail {
            Err(RbError::IsEmpty)
        } else {
            let elem = self.rb[self.tail];
            self.tail = (self.tail + 1) % CAP;
            Ok(elem)
        }
    }

    pub fn len(&self) -> usize {
        (self.head + CAP - self.tail) % CAP
    }

    pub fn capacity(&self) -> usize {
        CAP
    }

    pub fn peek_head(&self) -> T {
        self.rb[(self.head + CAP - 1) % CAP]
    }

    pub fn iter(&self) -> RingBufferIterator<T, CAP> {
        RingBufferIterator {
            rb: self,
            tail: self.tail,
        }
    }
}

impl<'a, T, const CAP: usize> Iterator for RingBufferIterator<'a, T, CAP> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rb.head == self.tail {
            None
        } else {
            let elem = &self.rb.rb[self.tail];
            self.tail = (self.tail + 1) % CAP;
            Some(elem)
        }
    }
}

pub struct RingBufferIterator<'a, T, const CAP: usize> {
    rb: &'a RingBuffer<T, CAP>,
    tail: usize,
}

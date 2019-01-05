use core::iter::FromIterator;
use core::slice;
use std::fmt;
use std::marker::PhantomData;

pub struct TypedVec<T>(Vec<T>);

pub struct TypedIndex<T> {
    index: usize,
    phantom: PhantomData<T>,
}

impl<T> Copy for TypedIndex<T> {}

impl<T> Clone for TypedIndex<T> {
    fn clone(&self) -> Self {
        TypedIndex::new(self.index)
    }
}

impl<T> PartialEq for TypedIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> TypedIndex<T> {
    pub fn new(index: usize) -> Self {
        TypedIndex {
            index,
            phantom: PhantomData,
        }
    }
}

impl<T> fmt::Display for TypedIndex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T> fmt::Debug for TypedIndex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T> TypedVec<T> {
    pub fn new() -> Self {
        TypedVec(Vec::new())
    }

    pub fn insert(&mut self, item: T) -> TypedIndex<T> {
        self.0.push(item);
        TypedIndex {
            index: self.0.len() - 1,
            phantom: PhantomData,
        }
    }

    pub fn get(&self, index: TypedIndex<T>) -> Option<&T> {
        self.0.get(index.index)
    }

    pub fn get_mut(&mut self, index: TypedIndex<T>) -> Option<&mut T> {
        self.0.get_mut(index.index)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            index: 0,
            inner: self.0.iter(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Iter<'a, T: 'a> {
    index: usize,
    inner: slice::Iter<'a, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (TypedIndex<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(value) => {
                let idx = TypedIndex {
                    index: self.index,
                    phantom: PhantomData,
                };
                self.index += 1;
                Some((idx, value))
            }
            None => None,
        }
    }
}

impl<T> IntoIterator for TypedVec<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> FromIterator<T> for TypedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = TypedVec::new();
        for i in iter {
            c.insert(i);
        }
        c
    }
}

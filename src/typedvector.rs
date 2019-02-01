use core::iter::*;
use core::slice;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Default)]
pub struct TypedVec<T>(Vec<T>);

pub struct TypedIndex<T> {
    index: usize,
    phantom: PhantomData<T>,
}

impl<T> Hash for TypedIndex<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> Eq for TypedIndex<T> {}

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

    pub fn into(self) -> usize {
        self.index
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

    pub fn remove(&mut self, index: TypedIndex<T>) -> T {
        self.0.remove(index.index)
    }

    pub fn get(&self, index: TypedIndex<T>) -> Option<&T> {
        self.0.get(index.index)
    }

    pub fn get_mut(&mut self, index: TypedIndex<T>) -> Option<&mut T> {
        self.0.get_mut(index.index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            index: 0,
            inner: self.0.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            index: 0,
            inner: self.0.iter_mut(),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.0
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

#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
    index: usize,
    inner: slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
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

#![doc = include_str!("../README.md")]

use std::ptr::NonNull;

pub struct Pass<T> {
    inner: NonNull<PassInner<T>>,
}

impl<T: Default> Default for Pass<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Pass<T> {
    pub fn new(data: T) -> Self {
        let data_ref = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(data))) };

        let inner = PassInner {
            previous: None,
            next: None,
            data: Some(data_ref),
        };

        Self {
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(inner))) },
        }
    }

    pub fn deref<'a>(&'a self) -> Option<&'a T> {
        unsafe { Some(self.inner.as_ref().data?.as_ref()) }
    }

    pub fn deref_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        unsafe { Some(self.inner.as_ref().data?.as_mut()) }
    }

    pub fn pass(&mut self) -> Option<Self> {
        let data_ref = unsafe { self.inner.as_mut().data.take()? };

        let next_inner = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(PassInner {
                previous: Some(self.inner),
                next: None,
                data: Some(data_ref),
            })))
        };

        unsafe { self.inner.as_mut().next = Some(next_inner) };

        let next = Self { inner: next_inner };

        Some(next)
    }
}

impl<T> Drop for Pass<T> {
    fn drop(&mut self) {
        unsafe {
            if self.inner.as_ref().data.is_some() {
                // we are the last reference in the chain.

                if let Some(mut previous) = self.inner.as_ref().previous {
                    previous.as_mut().data = self.inner.as_ref().data;
                } else {
                    // we are also the first reference, so drop the data as well.
                    self.inner.as_ref().data.unwrap().as_ptr().drop_in_place();
                }
            } else {
                self.inner.as_ref().next.unwrap().as_mut().previous = self.inner.as_ref().previous;
            }
        }
    }
}

struct PassInner<T> {
    previous: Option<NonNull<PassInner<T>>>,
    next: Option<NonNull<PassInner<T>>>,
    data: Option<NonNull<T>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_deref() {
        let data_ref = Pass::new(40);
        assert_eq!(data_ref.deref(), Some(&40));
    }

    #[test]
    fn create_and_deref_mut() {
        let mut data_ref = Pass::new(40);
        *data_ref.deref_mut().unwrap() = 80;
        assert_eq!(data_ref.deref(), Some(&80));
    }

    #[test]
    fn pass() {
        let mut ref_one = Pass::new(40);
        let ref_two = ref_one.pass().unwrap();

        assert_eq!(ref_one.deref(), None);
        assert_eq!(ref_two.deref(), Some(&40));
    }

    #[test]
    fn pass_back() {
        let mut ref_one = Pass::new(40);

        {
            let ref_two = ref_one.pass().unwrap();
            assert_eq!(ref_two.deref(), Some(&40));
        }

        assert_eq!(ref_one.deref(), Some(&40));
    }

    #[test]
    fn pass_and_drop() {
        let mut ref_one = Pass::new(40);

        let ref_two = ref_one.pass().unwrap();
        std::mem::drop(ref_one);

        assert_eq!(ref_two.deref(), Some(&40));
    }

    #[test]
    fn move_a_pass() {
        let mut ref_one_is_here = Some(Pass::new(40));
        let ref_one_will_be_here: Option<Pass<i32>>;

        {
            let ref_two = ref_one_is_here.as_mut().unwrap().pass().unwrap();
            assert_eq!(ref_two.deref(), Some(&40));

            ref_one_will_be_here = ref_one_is_here.take();
            assert_eq!(ref_one_will_be_here.as_ref().unwrap().deref(), None);
        }

        assert_eq!(ref_one_will_be_here.unwrap().deref(), Some(&40));
    }

    #[test]
    fn back_and_forth() {
        let mut one = Pass::new(40);

        {
            let two = one.pass().unwrap();
            assert_eq!(two.deref(), Some(&40));
        }

        assert_eq!(one.deref(), Some(&40));
    }
}

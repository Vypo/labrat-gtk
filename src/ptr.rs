use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc;

#[derive(Debug)]
pub struct Wrap<T> {
    this: RefCell<Weak<T>>,
    t: T,
}

impl<T> Wrap<T> {
    pub fn weak(&self) -> Weak<T> {
        self.this.borrow().clone()
    }
}

impl<T> Deref for Wrap<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.t
    }
}

impl<T> DerefMut for Wrap<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.t
    }
}

#[derive(Debug)]
pub struct Owned<T>(rc::Rc<Wrap<T>>);

impl<T> Default for Owned<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Owned<T> {
    pub fn new(t: T) -> Self {
        let owned = Self(rc::Rc::new(Wrap {
            this: RefCell::new(Weak::new()),
            t,
        }));

        let this = Owned::downgrade(&owned);
        owned.0.this.replace(this);

        owned
    }

    pub fn downgrade(owned: &Self) -> Weak<T> {
        Weak(rc::Rc::downgrade(&owned.0))
    }

    pub fn reference(owned: &Self) -> Ref<T> {
        Ref {
            _p: PhantomData,
            rc: owned.0.clone(),
        }
    }

    pub fn get_mut(owned: &mut Self) -> Option<&mut Wrap<T>> {
        rc::Rc::get_mut(&mut owned.0)
    }
}

impl<T> Deref for Owned<T> {
    type Target = Wrap<T>;

    fn deref(&self) -> &Wrap<T> {
        &self.0
    }
}

#[derive(Debug)]
pub struct Weak<T>(rc::Weak<Wrap<T>>);

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Weak<T> {
    pub fn new() -> Self {
        Self(rc::Weak::new())
    }

    pub fn upgrade(&self) -> Option<Ref<T>> {
        self.0.upgrade().map(|rc| Ref {
            _p: PhantomData,
            rc,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Ref<'a, T> {
    _p: PhantomData<&'a Weak<T>>,
    rc: rc::Rc<Wrap<T>>,
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = Wrap<T>;

    fn deref(&self) -> &Wrap<T> {
        &self.rc
    }
}

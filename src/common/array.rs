// This code is adapted from the Array Rust library
// Source: https://github.com/bluss/Array
// Original license: MIT/Apache-2.0

use std::any::Any;
use std::cmp;
use std::error::Error;
use std::fmt;
use std::io;
use std::iter;
use std::mem;
use std::mem::ManuallyDrop;
use std::ops::{Bound, Deref, DerefMut, RangeBounds};
use std::ptr;
use std::slice;

// extra traits
use std::borrow::{Borrow, BorrowMut};
use std::hash::{Hash, Hasher};

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

macro_rules! assert_capacity_limit {
    ($cap:expr) => {
        if std::mem::size_of::<usize>() > std::mem::size_of::<usize>() {
            if $cap > usize::MAX as usize {
                #[cfg(not(target_pointer_width = "16"))]
                panic!("Array: largest supported capacity is u32::MAX");
                #[cfg(target_pointer_width = "16")]
                panic!("Array: largest supported capacity is u16::MAX");
            }
        }
    };
}

macro_rules! assert_capacity_limit_const {
    ($cap:expr) => {
        if std::mem::size_of::<usize>() > std::mem::size_of::<usize>() {
            if $cap > usize::MAX as usize {
                [/*Array: largest supported capacity is u32::MAX*/][$cap]
            }
        }
    }
}

/// Error value indicating insufficient capacity
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapacityError<T = ()> {
    element: T,
}

impl<T> CapacityError<T> {
    /// Create a new `CapacityError` from `element`.
    pub const fn new(element: T) -> CapacityError<T> {
        CapacityError { element: element }
    }

    /// Extract the overflowing element
    pub fn element(self) -> T {
        self.element
    }

    /// Convert into a `CapacityError` that does not carry an element.
    pub fn simplify(self) -> CapacityError {
        CapacityError { element: () }
    }
}

const CAPERROR: &'static str = "insufficient capacity";

/// Requires `features="std"`.
impl<T: Any> Error for CapacityError<T> {}

impl<T> fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", CAPERROR)
    }
}

impl<T> fmt::Debug for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", "CapacityError", CAPERROR)
    }
}

#[derive(Copy, Clone)]
pub struct Array<T, const CAP: usize>
where
    T: Copy + Default,
{
    xs: [T; CAP],
    len: usize,
}

impl<T: Default + Copy, const CAP: usize> Array<T, CAP> {
    /// Capacity
    const CAPACITY: usize = CAP;

    pub fn new() -> Array<T, CAP> {
        assert_capacity_limit_const!(CAP);
        Array {
            xs: [T::default(); CAP],
            len: 0,
        }
    }

    /// Create a new empty `Array`.
    ///
    /// The maximum capacity is given by the generic parameter `CAP`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::<_, 16>::new();
    /// array.push(1);
    /// array.push(2);
    /// assert_eq!(&array[..], &[1, 2]);
    /// assert_eq!(array.capacity(), 16);
    /// ```
    #[inline]
    #[track_caller]
    pub fn new_const(t: T, len: usize) -> Array<T, CAP> {
        assert_capacity_limit!(CAP);
        unsafe {
            Array {
                xs: [t; CAP],
                len: 0,
            }
        }
    }

    /// Return the number of elements in the `Array`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.len(), 2);
    /// ```
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= CAP);
        self.len = length;
    }

    pub fn as_ptr(&self) -> *const T {
        self.xs.as_ptr() as _
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.xs.as_mut_ptr() as _
    }

    /// Returns whether the `Array` is empty.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1]);
    /// array.pop();
    /// assert_eq!(array.is_empty(), true);
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the capacity of the `Array`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let array = Array::from([1, 2, 3]);
    /// assert_eq!(array.capacity(), 3);
    /// ```
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Return true if the `Array` is completely filled to its capacity, false otherwise.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::<_, 1>::new();
    /// assert!(!array.is_full());
    /// array.push(1);
    /// assert!(array.is_full());
    /// ```
    pub const fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Returns the capacity left in the `Array`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.remaining_capacity(), 1);
    /// ```
    pub const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Return a slice containing all elements of the vector.
    fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { slice::from_raw_parts(self.as_ptr(), len) }
    }

    /// Return a mutable slice containing all elements of the vector.
    fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }

    #[track_caller]
    pub fn push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        self.try_push(element)
    }

    pub fn try_push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        if self.len() < Self::CAPACITY {
            unsafe {
                self.push_unchecked(element);
            }
            Ok(())
        } else {
            Err(CapacityError::new(element))
        }
    }

    pub unsafe fn push_unchecked(&mut self, element: T) {
        let len = self.len();
        debug_assert!(len < Self::CAPACITY);
        ptr::write(self.as_mut_ptr().add(len), element);
        self.set_len(len + 1);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let new_len = self.len() - 1;
            self.set_len(new_len);
            Some(ptr::read(self.as_ptr().add(new_len)))
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0)
    }

    pub fn truncate(&mut self, new_len: usize) {
        unsafe {
            let len = self.len();
            if new_len < len {
                self.set_len(new_len);
                let tail = slice::from_raw_parts_mut(self.as_mut_ptr().add(new_len), len - new_len);
                ptr::drop_in_place(tail);
            }
        }
    }

    /// Get pointer to where element at `index` would be
    pub unsafe fn get_unchecked_ptr(&mut self, index: usize) -> *mut T {
        self.as_mut_ptr().add(index)
    }

    /// Insert `element` at position `index`.
    ///
    /// Shift up all elements after `index`.
    ///
    /// It is an error if the index is greater than the length or if the
    /// Array is full.
    ///
    /// ***Panics*** if the array is full or the `index` is out of bounds. See
    /// `try_insert` for fallible version.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::<_, 2>::new();
    ///
    /// array.insert(0, "x");
    /// array.insert(0, "y");
    /// assert_eq!(&array[..], &["y", "x"]);
    ///
    /// ```
    #[track_caller]
    pub fn insert(&mut self, index: usize, element: T) {
        self.try_insert(index, element).unwrap()
    }

    /// Insert `element` at position `index`.
    ///
    /// Shift up all elements after `index`; the `index` must be less than
    /// or equal to the length.
    ///
    /// Returns an error if vector is already at full capacity.
    ///
    /// ***Panics*** `index` is out of bounds.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::<_, 2>::new();
    ///
    /// assert!(array.try_insert(0, "x").is_ok());
    /// assert!(array.try_insert(0, "y").is_ok());
    /// assert!(array.try_insert(0, "z").is_err());
    /// assert_eq!(&array[..], &["y", "x"]);
    ///
    /// ```
    pub fn try_insert(&mut self, index: usize, element: T) -> Result<(), CapacityError<T>> {
        if index > self.len() {
            panic!(
                "try_insert: index out of bounds: the len is {} but the index is {}",
                self.len(),
                index
            );
        }
        if self.len() == self.capacity() {
            return Err(CapacityError::new(element));
        }
        let len = self.len();

        // follows is just like Vec<T>
        unsafe {
            // infallible
            // The spot to put the new value
            {
                let p: *mut _ = self.get_unchecked_ptr(index);
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                ptr::copy(p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
        Ok(())
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This operation is O(1).
    ///
    /// Return the *element* if the index is in bounds, else panic.
    ///
    /// ***Panics*** if the `index` is out of bounds.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_remove(0), 1);
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_remove(1), 2);
    /// assert_eq!(&array[..], &[3]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.swap_pop(index).unwrap_or_else(|| {
            panic!(
                "swap_remove: index out of bounds: the len is {} but the index is {}",
                self.len(),
                index
            )
        })
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This is a checked version of `.swap_remove`.  
    /// This operation is O(1).
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_pop(0), Some(1));
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_pop(10), None);
    /// ```
    pub fn swap_pop(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            return None;
        }
        self.swap(index, len - 1);
        self.pop()
    }

    /// Remove the element at `index` and shift down the following elements.
    ///
    /// The `index` must be strictly less than the length of the vector.
    ///
    /// ***Panics*** if the `index` is out of bounds.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3]);
    ///
    /// let removed_elt = array.remove(0);
    /// assert_eq!(removed_elt, 1);
    /// assert_eq!(&array[..], &[2, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        self.pop_at(index).unwrap_or_else(|| {
            panic!(
                "remove: index out of bounds: the len is {} but the index is {}",
                self.len(),
                index
            )
        })
    }

    /// Remove the element at `index` and shift down the following elements.
    ///
    /// This is a checked version of `.remove(index)`. Returns `None` if there
    /// is no element at `index`. Otherwise, return the element inside `Some`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3]);
    ///
    /// assert!(array.pop_at(0).is_some());
    /// assert_eq!(&array[..], &[2, 3]);
    ///
    /// assert!(array.pop_at(2).is_none());
    /// assert!(array.pop_at(10).is_none());
    /// ```
    pub fn pop_at(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            None
        } else {
            self.drain(index..index + 1).next()
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns false.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut array = Array::from([1, 2, 3, 4]);
    /// array.retain(|x| *x & 1 != 0 );
    /// assert_eq!(&array[..], &[1, 3]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
        T: Default + Copy,
    {
        // Check the implementation of
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain
        // for safety arguments (especially regarding panics in f and when
        // dropping elements). Implementation closely mirrored here.

        let original_len = self.len();
        unsafe { self.set_len(0) };

        struct BackshiftOnDrop<'a, T: Default + Copy, const CAP: usize> {
            v: &'a mut Array<T, CAP>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T: Default + Copy, const CAP: usize> Drop for BackshiftOnDrop<'_, T, CAP> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v
                                .as_mut_ptr()
                                .add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        #[inline(always)]
        fn process_one<
            F: FnMut(&mut T) -> bool,
            T: Default + Copy,
            const CAP: usize,
            const DELETED: bool,
        >(
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T, CAP>,
        ) -> bool {
            let cur = unsafe { g.v.as_mut_ptr().add(g.processed_len) };
            if !f(unsafe { &mut *cur }) {
                g.processed_len += 1;
                g.deleted_cnt += 1;
                unsafe { ptr::drop_in_place(cur) };
                return false;
            }
            if DELETED {
                unsafe {
                    let hole_slot = cur.sub(g.deleted_cnt);
                    ptr::copy_nonoverlapping(cur, hole_slot, 1);
                }
            }
            g.processed_len += 1;
            true
        }

        // Stage 1: Nothing was deleted.
        while g.processed_len != original_len {
            if !process_one::<F, T, CAP, false>(&mut f, &mut g) {
                break;
            }
        }

        // Stage 2: Some elements were deleted.
        while g.processed_len != original_len {
            process_one::<F, T, CAP, true>(&mut f, &mut g);
        }

        drop(g);
    }

    // /// Returns the remaining spare capacity of the vector as a slice of
    // /// `MaybeUninit<T>`.
    // ///
    // /// The returned slice can be used to fill the vector with data (e.g. by
    // /// reading from a file) before marking the data as initialized using the
    // /// [`set_len`] method.
    // ///
    // /// [`set_len`]: Array::set_len
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use games_rs::common::array::Array;
    // ///
    // /// // Allocate vector big enough for 10 elements.
    // /// let mut v: Array<i32, 10> = Array::new();
    // ///
    // /// // Fill in the first 3 elements.
    // /// let uninit = v.spare_capacity_mut();
    // /// uninit[0].write(0);
    // /// uninit[1].write(1);
    // /// uninit[2].write(2);
    // ///
    // /// // Mark the first 3 elements of the vector as being initialized.
    // /// unsafe {
    // ///     v.set_len(3);
    // /// }
    // ///
    // /// assert_eq!(&v[..], &[0, 1, 2]);
    // /// ```
    // pub fn spare_capacity_mut(&mut self) -> &mut [T] {
    //     let len = self.len();
    //     &mut self.xs[len..]
    // }

    /// Copy all elements from the slice and append to the `Array`.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut vec: Array<usize, 10> = Array::new();
    /// vec.push(1);
    /// vec.try_extend_from_slice(&[2, 3]).unwrap();
    /// assert_eq!(&vec[..], &[1, 2, 3]);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if the capacity left (see
    /// [`remaining_capacity`]) is smaller then the length of the provided
    /// slice.
    ///
    /// [`remaining_capacity`]: #method.remaining_capacity
    pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), CapacityError>
    where
        T: Copy,
    {
        if self.remaining_capacity() < other.len() {
            return Err(CapacityError::new(()));
        }

        let self_len = self.len();
        let other_len = other.len();

        unsafe {
            let dst = self.get_unchecked_ptr(self_len);
            ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len);
            self.set_len(self_len + other_len);
        }
        Ok(())
    }

    /// Create a draining iterator that removes the specified range in the vector
    /// and yields the removed items from start to end. The element range is
    /// removed even if the iterator is not consumed until the end.
    ///
    /// Note: It is unspecified how many elements are removed from the vector,
    /// if the `Drain` value is leaked.
    ///
    /// **Panics** if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut v1 = Array::from([1, 2, 3]);
    /// let v2: Array<_, 3> = v1.drain(0..2).collect();
    /// assert_eq!(&v1[..], &[3]);
    /// assert_eq!(&v2[..], &[1, 2]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Drain<T, CAP>
    where
        R: RangeBounds<usize>,
    {
        // Memory safety
        //
        // When the Drain is first created, it shortens the length of
        // the source vector to make sure no uninitialized or moved-from elements
        // are accessible at all if the Drain's destructor never gets to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of the vec is copied back to cover
        // the hole, and the vector length is restored to the new length.
        //
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i.saturating_add(1),
        };
        let end = match range.end_bound() {
            Bound::Excluded(&j) => j,
            Bound::Included(&j) => j.saturating_add(1),
            Bound::Unbounded => len,
        };
        self.drain_range(start, end)
    }

    fn drain_range(&mut self, start: usize, end: usize) -> Drain<T, CAP> {
        let len = self.len();

        // bounds check happens here (before length is changed!)
        let range_slice: *const _ = &self[start..end];

        // Calling `set_len` creates a fresh and thus unique mutable references, making all
        // older aliases we created invalid. So we cannot call that function.
        self.len = start;

        unsafe {
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: (*range_slice).iter(),
                vec: self as *mut _,
            }
        }
    }

    /// Return the inner fixed size array, if it is full to its capacity.
    ///
    /// Return an `Ok` value with the array if length equals capacity,
    /// return an `Err` with self otherwise.
    pub fn into_inner(self) -> Result<[T; CAP], Self> {
        if self.len() < self.capacity() {
            Err(self)
        } else {
            unsafe { Ok(self.into_inner_unchecked()) }
        }
    }

    /// Return the inner fixed size array.
    ///
    /// Safety:
    /// This operation is safe if and only if length equals capacity.
    pub unsafe fn into_inner_unchecked(self) -> [T; CAP] {
        debug_assert_eq!(self.len(), self.capacity());
        let self_ = ManuallyDrop::new(self);
        let array = ptr::read(self_.as_ptr() as *const [T; CAP]);
        array
    }

    /// Returns the Array, replacing the original with a new empty Array.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let mut v = Array::from([0, 1, 2, 3]);
    /// assert_eq!([0, 1, 2, 3], v.take().into_inner().unwrap());
    /// assert!(v.is_empty());
    /// ```
    pub fn take(&mut self) -> Self {
        mem::replace(self, Self::new())
    }
}

impl<T: Default + Copy, const CAP: usize> Deref for Array<T, CAP> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> DerefMut for Array<T, CAP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> From<[T; CAP]> for Array<T, CAP> {
    /// Create an `Array` from a fixed size array.
    ///
    /// ```
    /// use games_rs::common::array::Array;
    ///
    /// let array: Array<_, 4> = Array::from([1, 2, 3, 4]);
    /// assert_eq!(array.len(), 4);
    /// assert_eq!(array.capacity(), 4);
    /// ```
    fn from(arr: [T; CAP]) -> Self {
        Array { xs: arr, len: CAP }
    }
}

/// Try to create an `Array` from a slice. This will return an error if the slice was too big to
/// fit.
///
/// ```
/// use games_rs::common::array::Array;
/// use std::convert::TryInto as _;
///
/// let array: Array<_, 4> = (&[1, 2, 3] as &[_]).try_into().unwrap();
/// assert_eq!(array.len(), 3);
/// assert_eq!(array.capacity(), 4);
/// ```
impl<T: Default + Copy, const CAP: usize> std::convert::TryFrom<&[T]> for Array<T, CAP>
where
    T: Clone,
{
    type Error = CapacityError;

    fn try_from(slice: &[T]) -> Result<Self, Self::Error> {
        if Self::CAPACITY < slice.len() {
            Err(CapacityError::new(()))
        } else {
            let mut array = Self::new();
            array.extend_from_slice(slice);
            Ok(array)
        }
    }
}

/// Iterate the `Array` with references to each element.
///
/// ```
/// use games_rs::common::array::Array;
///
/// let array = Array::from([1, 2, 3]);
///
/// for elt in &array {
///     // ...
/// }
/// ```
impl<'a, T: Default + Copy, const CAP: usize> IntoIterator for &'a Array<T, CAP> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterate the `Array` with mutable references to each element.
///
/// ```
/// use games_rs::common::array::Array;
///
/// let mut array = Array::from([1, 2, 3]);
///
/// for elt in &mut array {
///     // ...
/// }
/// ```
impl<'a, T: Default + Copy, const CAP: usize> IntoIterator for &'a mut Array<T, CAP> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Iterate the `Array` with each element by value.
///
/// The vector is consumed by this operation.
///
/// ```
/// use games_rs::common::array::Array;
///
/// for elt in Array::from([1, 2, 3]) {
///     // ...
/// }
/// ```
impl<T: Default + Copy, const CAP: usize> IntoIterator for Array<T, CAP> {
    type Item = T;
    type IntoIter = IntoIter<T, CAP>;
    fn into_iter(self) -> IntoIter<T, CAP> {
        IntoIter { index: 0, v: self }
    }
}

/// By-value iterator for `Array`.
pub struct IntoIter<T: Default + Copy, const CAP: usize> {
    index: usize,
    v: Array<T, CAP>,
}
impl<T: Default + Copy, const CAP: usize> IntoIter<T, CAP> {
    /// Returns the remaining items of this iterator as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.v[self.index..]
    }

    /// Returns the remaining items of this iterator as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.v[self.index..]
    }
}

impl<T: Default + Copy, const CAP: usize> Iterator for IntoIter<T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.index == self.v.len() {
            None
        } else {
            unsafe {
                let index = self.index;
                self.index = index + 1;
                Some(ptr::read(self.v.get_unchecked_ptr(index)))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len() - self.index;
        (len, Some(len))
    }
}

impl<T: Default + Copy, const CAP: usize> DoubleEndedIterator for IntoIter<T, CAP> {
    fn next_back(&mut self) -> Option<T> {
        if self.index == self.v.len() {
            None
        } else {
            unsafe {
                let new_len = self.v.len() - 1;
                self.v.set_len(new_len);
                Some(ptr::read(self.v.get_unchecked_ptr(new_len)))
            }
        }
    }
}

impl<T: Default + Copy, const CAP: usize> ExactSizeIterator for IntoIter<T, CAP> {}

impl<T: Default + Copy, const CAP: usize> Drop for IntoIter<T, CAP> {
    fn drop(&mut self) {
        // panic safety: Set length to 0 before dropping elements.
        let index = self.index;
        let len = self.v.len();
        unsafe {
            self.v.set_len(0);
            let elements = slice::from_raw_parts_mut(self.v.get_unchecked_ptr(index), len - index);
            ptr::drop_in_place(elements);
        }
    }
}

impl<T: Default + Copy, const CAP: usize> Clone for IntoIter<T, CAP>
where
    T: Clone,
{
    fn clone(&self) -> IntoIter<T, CAP> {
        let mut v = Array::new();
        v.extend_from_slice(&self.v[self.index..]);
        v.into_iter()
    }
}

impl<T: Default + Copy, const CAP: usize> fmt::Debug for IntoIter<T, CAP>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(&self.v[self.index..]).finish()
    }
}

/// A draining iterator for `Array`.
pub struct Drain<'a, T: Default + Copy, const CAP: usize> {
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: slice::Iter<'a, T>,
    vec: *mut Array<T, CAP>,
}

unsafe impl<'a, T: Default + Copy + Sync, const CAP: usize> Sync for Drain<'a, T, CAP> {}
unsafe impl<'a, T: Default + Copy + Send, const CAP: usize> Send for Drain<'a, T, CAP> {}

impl<'a, T: Default + Copy, const CAP: usize> Iterator for Drain<'a, T, CAP> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter
            .next()
            .map(|elt| unsafe { ptr::read(elt as *const _) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Default + Copy, const CAP: usize> DoubleEndedIterator for Drain<'a, T, CAP> {
    fn next_back(&mut self) -> Option<T> {
        self.iter
            .next_back()
            .map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<'a, T: Default + Copy, const CAP: usize> ExactSizeIterator for Drain<'a, T, CAP> {}

impl<'a, T: Default + Copy, const CAP: usize> Drop for Drain<'a, T, CAP> {
    fn drop(&mut self) {
        // len is currently 0 so panicking while dropping will not cause a double drop.

        // exhaust self first
        while let Some(_) = self.next() {}

        if self.tail_len > 0 {
            unsafe {
                let source_vec = &mut *self.vec;
                // memmove back untouched tail, update to new length
                let start = source_vec.len();
                let tail = self.tail_start;
                let ptr = source_vec.as_mut_ptr();
                ptr::copy(ptr.add(tail), ptr.add(start), self.tail_len);
                source_vec.set_len(start + self.tail_len);
            }
        }
    }
}

impl<T: Default + Copy, const CAP: usize> Borrow<[T]> for Array<T, CAP> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> BorrowMut<[T]> for Array<T, CAP> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> AsRef<[T]> for Array<T, CAP> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> AsMut<[T]> for Array<T, CAP> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> PartialEq for Array<T, CAP>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Default + Copy, const CAP: usize> PartialEq<[T]> for Array<T, CAP>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: Default + Copy, const CAP: usize> Eq for Array<T, CAP> where T: Eq {}

impl<T: Default + Copy, const CAP: usize> PartialOrd for Array<T, CAP>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Default + Copy, const CAP: usize> Ord for Array<T, CAP>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T: Default + Copy, const CAP: usize> Hash for Array<T, CAP>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl<T: Default + Copy, const CAP: usize> Serialize for Array<T, CAP>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

impl<'de, T: Default + Copy, const CAP: usize> Deserialize<'de> for Array<T, CAP>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<T> = Vec::deserialize(deserializer)?;
        let mut array = Array::<T, CAP>::new();
        for item in vec {
            array.push(item).map_err(serde::de::Error::custom)?;
        }
        Ok(array)
    }
}

impl<T: Default + Copy, const CAP: usize> io::Write for Array<T, CAP>
where
    T: AsMut<[u8]> + Default + Copy,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut total_written = 0;
        let mut remaining_buf = buf;

        while !remaining_buf.is_empty() && !self.is_full() {
            let mut slot = T::default();
            let slot_buf = slot.as_mut();

            let to_write = cmp::min(remaining_buf.len(), slot_buf.len());
            slot_buf[..to_write].copy_from_slice(&remaining_buf[..to_write]);

            self.push(slot).unwrap();

            total_written += to_write;
            remaining_buf = &remaining_buf[to_write..];
        }

        Ok(total_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: Default + Copy, const CAP: usize> fmt::Debug for Array<T, CAP>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Default + Copy, const CAP: usize> Default for Array<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

struct ScopeExitGuard<T, Data, F>
where
    F: FnMut(&Data, &mut T),
{
    value: T,
    data: Data,
    f: F,
}

impl<T, Data, F> Drop for ScopeExitGuard<T, Data, F>
where
    F: FnMut(&Data, &mut T),
{
    fn drop(&mut self) {
        (self.f)(&self.data, &mut self.value)
    }
}

/// Extend the `Array` with an iterator.
///
/// ***Panics*** if extending the vector exceeds its capacity.
impl<T: Default + Copy, const CAP: usize> Extend<T> for Array<T, CAP> {
    /// Extend the `Array` with an iterator.
    ///
    /// ***Panics*** if extending the vector exceeds its capacity.
    #[track_caller]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        unsafe { self.extend_from_iter::<_, true>(iter) }
    }
}

#[inline(never)]
#[cold]
#[track_caller]
fn extend_panic() {
    panic!("Array: capacity exceeded in extend/from_iter");
}

impl<T: Default + Copy, const CAP: usize> Array<T, CAP> {
    /// Extend the Array from the iterable.
    ///
    /// ## Safety
    ///
    /// Unsafe because if CHECK is false, the length of the input is not checked.
    /// The caller must ensure the length of the input fits in the capacity.
    #[track_caller]
    pub(crate) unsafe fn extend_from_iter<I, const CHECK: bool>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        let take = self.capacity() - self.len();
        let len = self.len();
        let mut ptr = raw_ptr_add(self.as_mut_ptr(), len);
        let end_ptr = raw_ptr_add(ptr, take);
        // Keep the length in a separate variable, write it back on scope
        // exit. To help the compiler with alias analysis and stuff.
        // We update the length to handle panic in the iteration of the
        // user's iterator, without dropping any elements on the floor.
        let mut guard = ScopeExitGuard {
            value: &mut self.len,
            data: len,
            f: move |&len, self_len| {
                **self_len = len;
            },
        };
        let mut iter = iterable.into_iter();
        loop {
            if let Some(elt) = iter.next() {
                if ptr == end_ptr && CHECK {
                    extend_panic();
                }
                debug_assert_ne!(ptr, end_ptr);
                if mem::size_of::<T>() != 0 {
                    ptr.write(elt);
                }
                ptr = raw_ptr_add(ptr, 1);
                guard.data += 1;
            } else {
                return; // success
            }
        }
    }

    /// Extend the Array with clones of elements from the slice;
    /// the length of the slice must be <= the remaining capacity in the Array.
    pub(crate) fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        let take = self.capacity() - self.len();
        debug_assert!(slice.len() <= take);
        unsafe {
            let slice = if take < slice.len() {
                &slice[..take]
            } else {
                slice
            };
            self.extend_from_iter::<_, false>(slice.iter().cloned());
        }
    }
}

/// Rawptr add but uses arithmetic distance for ZST
unsafe fn raw_ptr_add<T>(ptr: *mut T, offset: usize) -> *mut T {
    if mem::size_of::<T>() == 0 {
        // Special case for ZST
        ptr.cast::<u8>().wrapping_add(offset).cast::<T>()
    } else {
        ptr.add(offset)
    }
}

/// Create an `Array` from an iterator.
///
/// ***Panics*** if the number of elements in the iterator exceeds the Array's capacity.
impl<T: Default + Copy, const CAP: usize> iter::FromIterator<T> for Array<T, CAP> {
    /// Create an `Array` from an iterator.
    ///
    /// ***Panics*** if the number of elements in the iterator exceeds the Array's capacity.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut array = Array::new();
        array.extend(iter);
        array
    }
}

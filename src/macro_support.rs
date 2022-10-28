use core::cmp::Ordering;

/// `partial_cmp` implementation for a single `Option<T>` field.
#[inline]
fn __storage_option_partial_cmp<T>(x: &Option<T>, y: &Option<T>) -> Option<Ordering>
where
    T: PartialOrd<T>,
{
    match (x, y) {
        (Some(x), Some(y)) => x.partial_cmp(y),
        (None, Some(_)) => Some(Ordering::Greater),
        (Some(_), None) => Some(Ordering::Less),
        _ => Some(Ordering::Equal),
    }
}

/// `cmp` implementation for a single `Option<T>` field.
#[inline]
fn __storage_option_cmp<T>(x: &Option<T>, y: &Option<T>) -> Ordering
where
    T: Ord,
{
    match (x, y) {
        (Some(x), Some(y)) => x.cmp(y),
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

/// `partial_cmp` implementation over iterators which ensures that storage
/// ordering between `None` and `Some` is handled in a reasonable manner.
#[doc(hidden)]
#[allow(clippy::missing_inline_in_public_items)]
pub fn __storage_iterator_partial_cmp<T>(a: &[Option<T>], b: &[Option<T>]) -> Option<Ordering>
where
    T: PartialOrd<T>,
{
    let mut a = a.iter();
    let mut b = b.iter();

    loop {
        let x = match a.next() {
            None => {
                if b.next().is_none() {
                    return Some(Ordering::Equal);
                }

                return Some(Ordering::Less);
            }
            Some(x) => x,
        };

        let y = match b.next() {
            None => return Some(Ordering::Greater),
            Some(y) => y,
        };

        match __storage_option_partial_cmp(x, y) {
            Some(Ordering::Equal) => (),
            ordering => return ordering,
        }
    }
}

/// `cmp` implementation over iterators which ensures that storage ordering
/// between `None` and `Some` is handled in a reasonable manner.
#[doc(hidden)]
#[allow(clippy::missing_inline_in_public_items)]
pub fn __storage_iterator_cmp<T>(a: &[Option<T>], b: &[Option<T>]) -> Ordering
where
    T: Ord,
{
    let mut a = a.iter();
    let mut b = b.iter();

    loop {
        let x = match a.next() {
            None => {
                if b.next().is_none() {
                    return Ordering::Equal;
                }

                return Ordering::Less;
            }
            Some(x) => x,
        };

        let y = match b.next() {
            None => return Ordering::Greater,
            Some(y) => y,
        };

        match __storage_option_cmp(x, y) {
            Ordering::Equal => (),
            ordering => return ordering,
        }
    }
}

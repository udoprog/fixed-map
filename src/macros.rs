/// Implement a forwarding iterator.
#[macro_export]
macro_rules! iterator {
    (@$process:ident, {$ident:ident, $var:ident}, {$($lt:lifetime)?}, [$($param:tt),*], $key:ty, $value:ty => $item:ty) => {
        impl<$($lt ,)? $($param,)*> Iterator for $ident<$($lt ,)? $($param ,)*>
        where
            $key: $($lt +)? Key<$key, $value>,
            $value: $($lt)?,
        {
            type Item = $item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                iterator!(@$process self.iter.next())
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                iterator!(@$process self.iter.nth(n))
            }
        }

        impl<$($lt ,)? $($param,)*> DoubleEndedIterator for $ident<$($lt ,)? $($param,)*>
        where
            $key: $($lt +)? Key<$key, $value>,
            $value: $($lt)?,
            <<$key>::Storage as Storage<$key, $value>>::$var$(<$lt>)?: DoubleEndedIterator,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                iterator!(@$process self.iter.next_back())
            }

            #[inline]
            fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                iterator!(@$process self.iter.nth_back(n))
            }
        }

        impl<$($lt ,)? $($param,)*> ExactSizeIterator for $ident<$($lt ,)? $($param,)*>
        where
            $key: $($lt +)? Key<$key, $value>,
            $value: $($lt)?,
            <<$key>::Storage as Storage<$key, $value>>::$var$(<$lt>)?: ExactSizeIterator,
        {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }
    };

    (@identity $fn:expr) => {
        $fn
    };

    (@first $fn:expr) => {
        Some($fn?.0)
    };
}

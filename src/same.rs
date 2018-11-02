use std::sync::Arc;

pub trait Same {
    fn same_as(&self, other: &Self) -> bool;
}

impl<T> Same for Arc<T> {
    fn same_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

impl<T: Same> Same for [T] {
    fn same_as(&self, other: &Self) -> bool {
        assert_eq!(self.len(), other.len());

        self.iter()
            .zip(other.iter())
            .all(|(a, b)| a.same_as(b))
    }
}

impl<T: Same> Same for Vec<T> {
    fn same_as(&self, other: &Self) -> bool {
        assert_eq!(self.len(), other.len());

        self.iter()
            .zip(other.iter())
            .all(|(a, b)| a.same_as(b))
    }
}

impl<T1: Same, T2: Same> Same for (T1, T2) {
    fn same_as(&self, other: &Self) -> bool {
        self.0.same_as(&other.0) &&
        self.1.same_as(&other.1)
    }
}

impl<'a, T: Same> Same for &'a T {
    fn same_as(&self, other: &Self) -> bool {
        (*self).same_as(other)
    }
}

impl<T: Same> Same for Option<T> {
    fn same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (&Some(ref a), &Some(ref b)) => a.same_as(b),
            (&None, &None) => true,
            _ => false,
        }
    }
}

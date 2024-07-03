use futures::stream::{FusedStream, Stream};

use crate::{min_batch::MinBatch, min_batch_with_weight::MinBatchWithWeight};

pub trait MinBatchExt: Stream {
    fn min_batch<F>(self, min_batch_weight: usize, count_fn: F) -> MinBatch<Self, F, Self::Item>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> usize,
    {
        MinBatch::new(self, min_batch_weight, count_fn)
    }

    fn min_batch_with_weight<F>(
        self,
        min_batch_weight: usize,
        count_fn: F,
    ) -> MinBatchWithWeight<Self, F, Self::Item>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> usize,
    {
        MinBatchWithWeight::new(self, min_batch_weight, count_fn)
    }
}

// Implement the trait for all types that implement Stream
impl<T: ?Sized> MinBatchExt for T where T: Stream {}

impl<S: FusedStream, F, T> FusedStream for MinBatch<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() && self.items.is_empty()
    }
}

impl<S: FusedStream, F, T> FusedStream for MinBatchWithWeight<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() && self.items.is_empty()
    }
}

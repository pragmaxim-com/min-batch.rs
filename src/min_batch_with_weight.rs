use core::pin::Pin;
use core::task::{Context, Poll};
use futures::ready;
use futures::stream::{Fuse, Stream};
use futures::StreamExt;
use pin_project_lite::pin_project;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    #[derive(Debug)]
    pub struct MinBatchWithWeight<S, F, T> where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
        #[pin]
        pub(crate) stream: Fuse<S>,
        current_batch_weight: usize,
        pub(crate) items: Vec<S::Item>,
        min_batch_weight: usize,
        count_fn: F,
    }
}

impl<S, F, T> MinBatchWithWeight<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    pub fn new(stream: S, min_batch_weight: usize, count_fn: F) -> Self {
        MinBatchWithWeight {
            stream: stream.fuse(),
            current_batch_weight: 0,
            items: Vec::with_capacity(min_batch_weight),
            min_batch_weight,
            count_fn,
        }
    }
}

impl<S, F, T> Stream for MinBatchWithWeight<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    type Item = (Vec<S::Item>, usize);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut me = self.project();
        loop {
            match ready!(me.stream.as_mut().poll_next(cx)) {
                Some(item) => {
                    if me.items.is_empty() {
                        me.items.reserve(*me.min_batch_weight);
                    }
                    let new_count = (me.count_fn)(&item);
                    me.items.push(item);
                    *me.current_batch_weight += new_count;
                    if *me.current_batch_weight >= *me.min_batch_weight {
                        let batch_weight = *me.current_batch_weight;
                        *me.current_batch_weight = 0;
                        return Poll::Ready(Some((std::mem::take(me.items), batch_weight)));
                    }
                }
                None => {
                    let last = if me.items.is_empty() {
                        None
                    } else {
                        let batch_weight = *me.current_batch_weight;
                        *me.current_batch_weight = 0;
                        Some((std::mem::take(me.items), batch_weight))
                    };
                    return Poll::Ready(last);
                }
            }
        }
    }
}

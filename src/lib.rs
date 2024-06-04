use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project_lite::pin_project;
use tokio_stream::Stream;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    #[derive(Debug)]
    pub struct MinBatch<S, F, T> where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
        #[pin]
        stream: S,
        current_batch_size: usize,
        items: Vec<S::Item>,
        min_batch_size: usize,
        count_fn: F,
    }
}
impl<S, F, T> MinBatch<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    pub fn new(stream: S, batch_size: usize, count_fn: F) -> Self {
        MinBatch {
            stream,
            current_batch_size: 0,
            items: Vec::with_capacity(batch_size),
            min_batch_size: batch_size,
            count_fn,
        }
    }
}

impl<S, F, T> Stream for MinBatch<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    type Item = Vec<S::Item>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut me = self.as_mut().project();
        loop {
            match me.stream.as_mut().poll_next(cx) {
                Poll::Pending => break,
                Poll::Ready(Some(item)) => {
                    if me.items.is_empty() {
                        me.items.reserve(*me.min_batch_size);
                    }
                    let new_count = (me.count_fn)(&item);
                    me.items.push(item);
                    *me.current_batch_size += new_count;
                    if me.current_batch_size >= me.min_batch_size {
                        return Poll::Ready(Some(std::mem::take(me.items)));
                    }
                }
                Poll::Ready(None) => {
                    let last = if me.items.is_empty() {
                        None
                    } else {
                        Some(std::mem::take(me.items))
                    };

                    return Poll::Ready(last);
                }
            }
        }

        if !me.items.is_empty() {
            return Poll::Ready(Some(std::mem::take(me.items)));
        }

        Poll::Pending
    }
}

pub trait MinBatchExt: Stream {
    fn min_batch<F>(self, min_batch_size: usize, count_fn: F) -> MinBatch<Self, F, Self::Item>
    where
        Self: Sized,
        F: Fn(&Self::Item) -> usize,
    {
        MinBatch::new(self, min_batch_size, count_fn)
    }
}

// Implement the trait for all types that implement Stream
impl<T: ?Sized> MinBatchExt for T where T: Stream {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::collections::VecDeque;

    #[tokio::test]
    async fn test_batch_stream_of_vectors() {
        let mut queue: VecDeque<char> = ('a'..='z').collect();

        let batches: Vec<Vec<Vec<char>>> = tokio_stream::iter(1..=5)
            .map(|x| {
                (0..x)
                    .map(|_| queue.pop_front().unwrap())
                    .collect::<Vec<char>>()
            })
            .min_batch(3, |xs: &Vec<char>| xs.len())
            .collect()
            .await;

        // Verify the batches
        assert_eq!(batches.len(), 4);
        assert_eq!(batches[0], vec![vec!['a'], vec!['b', 'c']]); // collect first vector elements until the total count is >= 3
        assert_eq!(batches[1], vec![vec!['d', 'e', 'f']]); // the third vector has already 3 elements so lets move to the next one
        assert_eq!(batches[2], vec![vec!['g', 'h', 'i', 'j']]);
    }

    #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
    struct BlockOfTxs {
        name: char,
        txs_count: usize,
    }

    #[tokio::test]
    async fn test_batch_stream_of_blocks() {
        let mut queue: VecDeque<char> = ('a'..='z').collect();

        let batches: Vec<Vec<BlockOfTxs>> = tokio_stream::iter(1..=5)
            .map(|x| BlockOfTxs {
                name: queue.pop_front().unwrap(),
                txs_count: x,
            })
            .min_batch(3, |block: &BlockOfTxs| block.txs_count)
            .collect()
            .await;

        // Verify the batches
        assert_eq!(batches.len(), 4);
        // collect first two Blocks of Transactions until the total count of transactions is >= 3
        assert_eq!(
            batches[0],
            vec![
                BlockOfTxs {
                    name: 'a',
                    txs_count: 1
                },
                BlockOfTxs {
                    name: 'b',
                    txs_count: 2
                }
            ],
        );
        // the third Block has already 3 transactions so lets move to the next one
        assert_eq!(
            batches[1],
            vec![BlockOfTxs {
                name: 'c',
                txs_count: 3
            }],
        );
        assert_eq!(
            batches[2],
            vec![BlockOfTxs {
                name: 'd',
                txs_count: 4
            }],
        );
    }
}

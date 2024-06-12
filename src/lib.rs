//! An adapter that turns elements into a batch of minimal element count. Needed for efficient work
//! parallelization so that following tasks running in parallel are all processing at least
//! `min_batch_size` of elements to avoid context switching overhead of cpu intensive workloads.
//!
//! ## Usage
//!
//! Either as a standalone stream operator or directly as a combinator:
//!
//! ```rust
//! use futures::{stream, StreamExt};
//! use min_batch::MinBatchExt;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct BlockOfTxs {
//!     name: char,
//!     txs_count: usize,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!         let mut block_names: Vec<char> = vec!['a', 'b', 'c', 'd'];
//!         let min_batch_size = 3;
//!         let batches: Vec<Vec<BlockOfTxs>> = stream::iter(1..=4)
//!            .map(|x| BlockOfTxs {
//!                name: block_names[x - 1],
//!                txs_count: x,
//!            })
//!            .min_batch(min_batch_size, |block: &BlockOfTxs| block.txs_count)
//!            .collect()
//!            .await;
//!
//!            // Verify the batches
//!            assert_eq!(batches.len(), 3);
//!           
//!            // collect first two Blocks of Transactions until the total count of transactions is >= 3
//!            assert_eq!(
//!                batches[0],
//!                vec![
//!                    BlockOfTxs {
//!                        name: 'a',
//!                        txs_count: 1
//!                    },
//!                    BlockOfTxs {
//!                        name: 'b',
//!                        txs_count: 2
//!                    }
//!                ],
//!            );
//!           
//!            // the third Block has already 3 transactions so lets move to the next one
//!            assert_eq!(
//!                batches[1],
//!                vec![BlockOfTxs {
//!                    name: 'c',
//!                    txs_count: 3
//!                }],
//!            );
//!            assert_eq!(
//!                batches[2],
//!                vec![BlockOfTxs {
//!                    name: 'd',
//!                    txs_count: 4
//!                }],
//!            );
//! }
//! ```
//!

#[cfg(test)]
#[macro_use]
extern crate doc_comment;

#[cfg(test)]
doctest!("../README.md");

use core::pin::Pin;
use core::task::{Context, Poll};
use futures::ready;
use futures::stream::{Fuse, FusedStream, Stream};
use futures::StreamExt;
use pin_project_lite::pin_project;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    #[derive(Debug)]
    pub struct MinBatch<S, F, T> where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
        #[pin]
        stream: Fuse<S>,
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
    pub fn new(stream: S, min_batch_size: usize, count_fn: F) -> Self {
        MinBatch {
            stream: stream.fuse(),
            current_batch_size: 0,
            items: Vec::with_capacity(min_batch_size),
            min_batch_size,
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
            match ready!(me.stream.as_mut().poll_next(cx)) {
                Some(item) => {
                    if me.items.is_empty() {
                        me.items.reserve(*me.min_batch_size);
                    }
                    let new_count = (me.count_fn)(&item);
                    me.items.push(item);
                    *me.current_batch_size += new_count;
                    if me.current_batch_size >= me.min_batch_size {
                        *me.current_batch_size = 0;
                        return Poll::Ready(Some(std::mem::take(me.items)));
                    }
                }
                None => {
                    let last = if me.items.is_empty() {
                        None
                    } else {
                        *me.current_batch_size = 0;
                        Some(std::mem::take(me.items))
                    };
                    return Poll::Ready(last);
                }
            }
        }
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

impl<S: FusedStream, F, T> FusedStream for MinBatch<S, F, T>
where
    S: Stream<Item = T>,
    F: Fn(&T) -> usize,
{
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() & self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{stream, StreamExt};
    use std::collections::VecDeque;

    #[tokio::test]
    async fn test_batch_stream_of_vectors() {
        let mut queue: VecDeque<char> = ('a'..='z').collect();

        let batches: Vec<Vec<Vec<char>>> = stream::iter(1..=5)
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

    #[derive(Debug, PartialEq, Eq)]
    struct BlockOfTxs {
        name: char,
        txs_count: usize,
    }

    #[tokio::test]
    async fn test_batch_stream_short() {
        let mut queue: VecDeque<char> = ('a'..='z').collect();

        let batches: Vec<Vec<Vec<char>>> = stream::once(async { 1 })
            .map(|x| {
                (0..x)
                    .map(|_| queue.pop_front().unwrap())
                    .collect::<Vec<char>>()
            })
            .min_batch(3, |xs: &Vec<char>| xs.len())
            .collect()
            .await;

        // Verify the batches
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], vec![vec!['a']]);
    }

    #[tokio::test]
    async fn test_batch_stream_of_blocks() {
        let mut queue: VecDeque<char> = ('a'..='z').collect();

        let batches: Vec<Vec<BlockOfTxs>> = tokio_stream::iter(1..=7)
            .map(|x| BlockOfTxs {
                name: queue.pop_front().unwrap(),
                txs_count: if x >= 4 { 1 } else { x },
            })
            .min_batch(3, |block: &BlockOfTxs| block.txs_count)
            .collect()
            .await;

        // Verify the batches
        assert_eq!(batches.len(), 4);
        // collect first Blocks of Transactions until the total count of transactions is >= 3
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
        // collect 3 more blocks with a single transaction
        assert_eq!(
            batches[2],
            vec![
                BlockOfTxs {
                    name: 'd',
                    txs_count: 1
                },
                BlockOfTxs {
                    name: 'e',
                    txs_count: 1
                },
                BlockOfTxs {
                    name: 'f',
                    txs_count: 1
                }
            ],
        );
        // and so on
        assert_eq!(
            batches[3],
            vec![BlockOfTxs {
                name: 'g',
                txs_count: 1
            }],
        );
    }
}

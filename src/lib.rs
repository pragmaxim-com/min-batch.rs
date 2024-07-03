//! An adapter that turns elements into a batch of minimal element count. Needed for efficient work
//! parallelization so that following tasks running in parallel are all processing at least
//! `min_batch_weight` of elements to avoid context switching overhead of cpu intensive workloads.
//!
//! ## Usage
//!
//! Either as a standalone stream operator or directly as a combinator:
//!
//! ```rust
//! use futures::{stream, StreamExt};
//! use min_batch::ext::MinBatchExt;
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
//!         let min_batch_weight = 3;
//!         let batches: Vec<Vec<BlockOfTxs>> = stream::iter(1..=4)
//!            .map(|x| BlockOfTxs {
//!                name: block_names[x - 1],
//!                txs_count: x,
//!            })
//!            .min_batch(min_batch_weight, |block: &BlockOfTxs| block.txs_count)
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

pub mod ext;
pub mod min_batch;
pub mod min_batch_with_weight;

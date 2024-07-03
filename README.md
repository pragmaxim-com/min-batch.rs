## min-batch

![Build status](https://github.com/pragmaxim-com/min-batch.rs/workflows/Rust/badge.svg)
[![Cargo](https://img.shields.io/crates/v/min-batch.svg)](https://crates.io/crates/min-batch)
[![Documentation](https://docs.rs/min-batch/badge.svg)](https://docs.rs/min-batch)

An adapter that turns elements into a batch and its weight is computed by given closure. 
It is needed for efficient work parallelization so that following tasks running in parallel 
are all processing a batch of at least `min_batch_weight` to avoid context switching overhead
of cpu intensive workloads. Otherwise we usually need to introduce some kind of publish/subscribe
model with dedicated long-running thread for each consumer, broadcasting messages to them and
establishing back-pressure through [barrier](https://docs.rs/tokio/latest/tokio/sync/struct.Barrier.html).

## Usage

There are 2 stream extension methods : 
  - `min_batch(min_batch_weight, fn_to_extract_weight)`
  - `min_batch_with_weight(min_batch_weight, fn_to_extract_weight)`

The elements are grouped into batches of minimal weight, possible returning the weight of a batch with it  

```rust
use futures::{stream, StreamExt};
use min_batch::ext::MinBatchExt;

#[derive(Debug, PartialEq, Eq)]
struct BlockOfTxs {
    name: char,
    txs_count: usize,
}

#[tokio::main]
async fn main() {
   let mut block_names: Vec<char> = vec!['a', 'b', 'c', 'd'];
   let min_batch_weight = 3;
   let batches: Vec<Vec<BlockOfTxs>> = 
       stream::iter(1..=4)
           .map(|x| BlockOfTxs {
               name: block_names[x - 1],
               txs_count: x,
           })
           .min_batch(min_batch_weight, |block: &BlockOfTxs| block.txs_count)
           .collect()
           .await;
   
   assert_eq!(batches.len(), 3);
   
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
```

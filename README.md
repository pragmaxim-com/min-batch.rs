## min-batch

![Build status](https://github.com/pragmaxim/min-batch/workflows/Rust/badge.svg)
[![Cargo](https://img.shields.io/crates/v/futures-batch.svg)](https://crates.io/crates/futures-batch)
[![Documentation](https://docs.rs/futures-batch/badge.svg)](https://docs.rs/futures-batch)

An adapter that turns elements into a batch of minimal element count. Needed for efficient work
parallelization so that following tasks running in parallel are all processing at least 
`min_batch_size` of elements to avoid context switching overhead of cpu intensive workloads.

## Usage

Either as a standalone stream operator or directly as a combinator.

```rust
use futures::{stream, StreamExt};
use min_batch::MinBatchExt;
#[derive(Debug, PartialEq, Eq)]
struct BlockOfTxs {
    name: char,
    txs_count: usize,
}
#[tokio::main]
async fn main() {
   let mut block_names: Vec<char> = vec!['a', 'b', 'c', 'd'];
   let min_match_size = 3;
   let batches: Vec<Vec<BlockOfTxs>> = stream::iter(1..=4)
   .map(|x| BlockOfTxs {
       name: block_names[x - 1],
       txs_count: x,
   })
   .min_batch(min_match_size, |block: &BlockOfTxs| block.txs_count)
   .collect()
   .await;
   // Verify the batches
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

## Credits

Thanks to [future-batch](https://github.com/mre/futures-batch) contributors for inspiration!!!

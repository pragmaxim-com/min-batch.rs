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

    #[tokio::test]
    async fn test_min_size_of_vectors() {
        let batches: Vec<Vec<Vec<char>>> = stream::iter(1..=10_005)
            .map(|_| (0..10).map(|_| 'a').collect::<Vec<char>>())
            .min_batch(100_000, |xs: &Vec<char>| xs.len())
            .collect()
            .await;

        // Verify the batches
        assert!(
            batches.len() == 2,
            "Expected batches length should not be {}",
            batches.len()
        );
        assert!(
            batches[0].len() == 10_000,
            "Expected batch length should not be {}",
            batches[0].len()
        );
        assert!(
            batches[1].len() == 5,
            "Expected batch length should not be {}",
            batches[1].len()
        );
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

    #[tokio::test]
    async fn test_batch_stream_of_blocks_with_size() {
        let mut queue: VecDeque<char> = ('a'..='z').collect();

        let batches: Vec<(Vec<BlockOfTxs>, usize)> = tokio_stream::iter(1..=7)
            .map(|x| BlockOfTxs {
                name: queue.pop_front().unwrap(),
                txs_count: if x >= 4 { 1 } else { x },
            })
            .min_batch_with_size(3, |block: &BlockOfTxs| block.txs_count)
            .collect()
            .await;

        // Verify the batches
        assert_eq!(batches.len(), 4);
        // collect first Blocks of Transactions until the total count of transactions is >= 3
        assert_eq!(
            batches[0],
            (
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
                3
            )
        );
        // the third Block has already 3 transactions so lets move to the next one
        assert_eq!(
            batches[1],
            (
                vec![BlockOfTxs {
                    name: 'c',
                    txs_count: 3
                }],
                3
            )
        );
        // collect 3 more blocks with a single transaction
        assert_eq!(
            batches[2],
            (
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
                3
            )
        );
        // and so on
        assert_eq!(
            batches[3],
            (
                vec![BlockOfTxs {
                    name: 'g',
                    txs_count: 1
                }],
                1
            )
        );
    }
}

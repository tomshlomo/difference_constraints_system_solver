pub struct PriorityQueueWithDuplicates<I: Hash + Eq, P: Ord> {
    queue: PriorityQueue<I, P>,
    counts: HashMap<I, usize>,
}

// #[cfg(test)]
// mod tests {
//     fn test_1() {
//         let x = PriorityQueueWithDuplicates::new();
//     }
// }

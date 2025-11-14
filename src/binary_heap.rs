pub struct Heap<K, T> {
    keys: Vec<K>,
    aux: Vec<T>,
    len: usize,
}

impl<K, T> Heap<K, T>
where
    K: Default + Clone + Copy + PartialOrd + Ord + PartialEq + Eq,
    T: Default + Clone + Copy,
{
    /// Inserts a dummy element, because the calculations of the indices of a 1-based binary heap
    /// are easier. Since the number of elements in the graph is usually known, a facility to
    /// preallocate the needed number of slots in the arrays is provided.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut keys = Vec::with_capacity(capacity);
        let mut aux = Vec::with_capacity(capacity);

        keys.push(K::default());
        aux.push(T::default());

        Self { keys, aux, len: 0 }
    }

    /// Returns the minimum element if it exists without removing it from the heap.
    pub fn min(&self) -> Option<(K, T)> {
        if self.len == 0 {
            None
        } else {
            Some((self.keys[1], self.aux[1]))
        }
    }

    /// Utility procedure to update the heap after inserting an element.
    fn ascend(&mut self, mut index: usize) {
        let mut parent = index >> 1;
        while parent > 0 {
            if self.keys[index] < self.keys[parent] {
                self.swap(index, parent);
            } else {
                break;
            }

            index = parent;
            parent >>= 1;
        }
    }

    /// Utility procedure to update the heap after removing an element.
    fn descend(&mut self, mut index: usize) {
        let mut left_child = index << 1;
        let mut right_child;
        let mut min_child;

        while left_child < self.keys.len() {
            right_child = left_child + 1;

            min_child = if right_child < self.keys.len()
                && self.keys[left_child] > self.keys[right_child]
            {
                right_child
            } else {
                left_child
            };

            if self.keys[index] <= self.keys[min_child] {
                break;
            }

            self.swap(index, min_child);
            index = min_child;
            left_child = index << 1;
        }
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.keys.swap(a, b);
        self.aux.swap(a, b);
    }

    /// Inserts an element into the heap.
    pub fn insert(&mut self, key: K, aux: T) {
        let new_index = self.keys.len();
        self.keys.push(key);
        self.aux.push(aux);
        self.ascend(new_index);
        self.len += 1;
    }

    /// Returns the minimum element if it exists and removes it from the queue.
    pub fn extract_min(&mut self) -> Option<(K, T)> {
        if self.len == 0 {
            return None;
        }

        let result = Some((self.keys[1], self.aux[1]));

        self.keys.swap_remove(1);
        self.aux.swap_remove(1);
        self.descend(1);

        self.len -= 1;

        result
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.keys.resize_with(1, Default::default);
        self.aux.resize_with(1, Default::default);
        self.len = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insertion_and_min() {
        let mut heap: Heap<usize, usize> = Heap::with_capacity(4);
        heap.insert(2, 2);
        heap.insert(1, 1);
        heap.insert(0, 0);
        let result = heap.min().unwrap();
        assert_eq!(0, result.0, "Key is incorrect.");
        assert_eq!(0, result.0, "Additional value is incorrect.");
        assert_eq!(heap.len, 3, "Size of the heap is incorrect.");
        assert!(!heap.is_empty(), "Heap should not be empty.");

        // Perform checks again after clearing.
        heap.clear();
        assert!(heap.is_empty(), "Heap should be empty.");
        heap.insert(0, 0);
        heap.insert(1, 1);
        heap.insert(2, 2);
        let result = heap.min().unwrap();
        assert_eq!(0, result.0, "Key is incorrect.");
        assert_eq!(0, result.0, "Additional value is incorrect.");
        assert_eq!(heap.len, 3, "Size of the heap is incorrect.");
        assert!(!heap.is_empty(), "Heap should not be empty.");
    }

    #[test]
    fn insertion_and_extract_min() {
        let mut heap: Heap<usize, usize> = Heap::with_capacity(4);
        heap.insert(2, 2);
        heap.insert(0, 0);
        heap.insert(1, 1);
        let result = heap.extract_min().unwrap();
        assert_eq!(0, result.0, "Key is incorrect.");
        assert_eq!(0, result.0, "Additional value is incorrect.");
        assert_eq!(heap.len, 2, "Size of the heap is incorrect.");
        heap.extract_min().unwrap();
        heap.extract_min().unwrap();
        assert!(heap.is_empty(), "Heap should be empty.")
    }
}

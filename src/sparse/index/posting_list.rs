// Modified for grayce/entity-linking: the max_next_weight propagation stops once an entry
// already has its new value. See provenance/README.md.

use std::cmp::max;

use crate::common::types::PointOffsetType;
use ordered_float::OrderedFloat;

use super::posting_list_common::{
    DEFAULT_MAX_NEXT_WEIGHT, PostingElement, PostingElementEx, PostingListIter,
};
use crate::sparse::common::types::DimWeight;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PostingList {
    /// List of the posting elements ordered by id
    pub elements: Vec<PostingElementEx>,
}

impl PostingList {
    #[cfg(test)]
    pub fn from(records: Vec<(PointOffsetType, DimWeight)>) -> PostingList {
        let mut posting_list = PostingBuilder::new();
        for (id, weight) in records {
            posting_list.add(id, weight);
        }
        posting_list.build()
    }

    /// Creates a new posting list with a single element.
    pub fn new_one(record_id: PointOffsetType, weight: DimWeight) -> PostingList {
        PostingList {
            elements: vec![PostingElementEx::new(record_id, weight)],
        }
    }

    pub fn delete(&mut self, record_id: PointOffsetType) {
        let index = self
            .elements
            .binary_search_by_key(&record_id, |e| e.record_id);
        if let Ok(found_index) = index {
            self.elements.remove(found_index);
            if let Some(last) = self.elements.last_mut() {
                last.max_next_weight = DEFAULT_MAX_NEXT_WEIGHT;
            }
            if found_index < self.elements.len() {
                self.propagate_max_next_weight_to_the_left(found_index);
            } else if !self.elements.is_empty() {
                self.propagate_max_next_weight_to_the_left(self.elements.len() - 1);
            }
        }
    }

    /// Upsert a posting element into the posting list.
    ///
    /// Worst case is adding a new element at the end of the list with a very large weight.
    /// This forces to propagate it as potential max_next_weight to all the previous elements.
    pub fn upsert(&mut self, posting_element: PostingElementEx) {
        // find insertion point in sorted posting list (most expensive operation for large posting list)
        let index = self
            .elements
            .binary_search_by_key(&posting_element.record_id, |e| e.record_id);

        let modified_index = match index {
            Ok(found_index) => {
                // Update existing element for the same id
                let element = &mut self.elements[found_index];
                if element.weight == posting_element.weight {
                    // no need to update anything
                    None
                } else {
                    // the structure of the posting list is not changed, no need to update max_next_weight
                    element.weight = posting_element.weight;
                    Some(found_index)
                }
            }
            Err(insert_index) => {
                // Insert new element by shifting elements to the right
                self.elements.insert(insert_index, posting_element);
                // the structure of the posting list is changed, need to update max_next_weight
                if insert_index == self.elements.len() - 1 {
                    // inserted at the end
                    Some(insert_index)
                } else {
                    // inserted in the middle - need to propagated max_next_weight from the right
                    Some(insert_index + 1)
                }
            }
        };
        // Propagate max_next_weight update to the previous entries
        if let Some(modified_index) = modified_index {
            self.propagate_max_next_weight_to_the_left(modified_index);
        }
    }

    /// Propagates `max_next_weight` from the entry at `up_to_index` to previous entries.
    ///
    /// Stops at the first entry, after the one right before `up_to_index`, whose
    /// `max_next_weight` already has the new value: every entry before it is then unchanged
    /// too, because each one's value is the previous one's value maxed with that entry's
    /// unchanged weight. The entry right before `up_to_index` is always written: it may be a
    /// newly inserted element, whose own weight its predecessors do not reflect yet.
    fn propagate_max_next_weight_to_the_left(&mut self, up_to_index: usize) {
        // used element at `up_to_index` as the starting point
        let starting_element = &self.elements[up_to_index];
        let mut max_next_weight = max(
            OrderedFloat(starting_element.max_next_weight),
            OrderedFloat(starting_element.weight),
        )
        .0;

        // propagate max_next_weight update to the previous entries
        for (distance, element) in self.elements[..up_to_index].iter_mut().rev().enumerate() {
            if distance > 0 && element.max_next_weight.to_bits() == max_next_weight.to_bits() {
                break;
            }
            // update max_next_weight for element
            element.max_next_weight = max_next_weight;
            max_next_weight = max_next_weight.max(element.weight);
        }
    }

    pub fn iter(&self) -> PostingListIterator<'_> {
        PostingListIterator::new(&self.elements)
    }
}

pub struct PostingBuilder {
    elements: Vec<PostingElementEx>,
}

impl Default for PostingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PostingBuilder {
    pub fn new() -> PostingBuilder {
        PostingBuilder {
            elements: Vec::new(),
        }
    }

    /// Add a new record to the posting list.
    pub fn add(&mut self, record_id: PointOffsetType, weight: DimWeight) {
        self.elements.push(PostingElementEx::new(record_id, weight));
    }

    /// Consume the builder and return the posting list.
    pub fn build(mut self) -> PostingList {
        // Sort by id
        self.elements.sort_unstable_by_key(|e| e.record_id);

        // Check for duplicates
        #[cfg(debug_assertions)]
        {
            if let Some([e, _]) = self
                .elements
                .array_windows()
                .find(|[a, b]| a.record_id == b.record_id)
            {
                panic!("Duplicate id {} in posting list", e.record_id);
            }
        }

        // Calculate the `max_next_weight` for all elements starting from the end
        let mut max_next_weight = f32::NEG_INFINITY;
        for element in self.elements.iter_mut().rev() {
            element.max_next_weight = max_next_weight;
            max_next_weight = max_next_weight.max(element.weight);
        }

        PostingList {
            elements: self.elements,
        }
    }
}

/// Iterator over posting list elements offering skipping abilities to avoid full iteration.
#[derive(Debug, Clone)]
pub struct PostingListIterator<'a> {
    pub elements: &'a [PostingElementEx],
    pub current_index: usize,
}

impl PostingListIter for PostingListIterator<'_> {
    #[inline]
    fn peek(&mut self) -> Option<PostingElementEx> {
        self.elements.get(self.current_index).cloned()
    }

    #[inline]
    fn last_id(&self) -> Option<PointOffsetType> {
        self.elements.last().map(|e| e.record_id)
    }

    fn element_size(&self) -> usize {
        size_of::<DimWeight>()
    }

    #[inline]
    fn skip_to(&mut self, record_id: PointOffsetType) -> Option<PostingElementEx> {
        self.skip_to(record_id)
    }

    #[inline]
    fn skip_to_end(&mut self) {
        self.skip_to_end();
    }

    #[inline]
    fn len_to_end(&self) -> usize {
        self.len_to_end()
    }

    #[inline]
    fn current_index(&self) -> usize {
        self.current_index
    }

    fn for_each_till_id<Ctx: ?Sized>(
        &mut self,
        id: PointOffsetType,
        ctx: &mut Ctx,
        mut f: impl FnMut(&mut Ctx, PointOffsetType, DimWeight),
    ) {
        let mut current_index = self.current_index;
        for element in &self.elements[current_index..] {
            if element.record_id > id {
                break;
            }
            f(ctx, element.record_id, element.weight);
            current_index += 1;
        }
        self.current_index = current_index;
    }

    fn reliable_max_next_weight() -> bool {
        true
    }

    fn into_std_iter(self) -> impl Iterator<Item = PostingElement> {
        self.elements.iter().cloned().map(PostingElement::from)
    }
}

impl<'a> PostingListIterator<'a> {
    pub fn new(elements: &'a [PostingElementEx]) -> PostingListIterator<'a> {
        PostingListIterator {
            elements,
            current_index: 0,
        }
    }

    /// Advances the iterator to the next element.
    pub fn advance(&mut self) {
        if self.current_index < self.elements.len() {
            self.current_index += 1;
        }
    }

    /// Returns the next element without advancing the iterator.
    pub fn peek(&self) -> Option<&PostingElementEx> {
        self.elements.get(self.current_index)
    }

    /// Returns the number of elements from the current position to the end of the list.
    pub fn len_to_end(&self) -> usize {
        self.elements.len() - self.current_index
    }

    /// Tries to find the element with ID == id and returns it.
    /// If the element is not found, the iterator is advanced to the next element with ID > id
    /// and None is returned.
    /// If the iterator is already at the end, None is returned.
    /// If the iterator skipped to the end, None is returned and current index is set to the length of the list.
    /// Uses binary search.
    pub fn skip_to(&mut self, id: PointOffsetType) -> Option<PostingElementEx> {
        // Check if we are already at the end
        if self.current_index >= self.elements.len() {
            return None;
        }

        // Use binary search to find the next element with ID > id
        let next_element =
            self.elements[self.current_index..].binary_search_by(|e| e.record_id.cmp(&id));

        match next_element {
            Ok(found_offset) => {
                self.current_index += found_offset;
                Some(self.elements[self.current_index].clone())
            }
            Err(insert_index) => {
                self.current_index += insert_index;
                None
            }
        }
    }

    /// Skips to the end of the posting list and returns None.
    pub fn skip_to_end(&mut self) -> Option<&PostingElementEx> {
        self.current_index = self.elements.len();
        None
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::sparse::index::posting_list_common::DEFAULT_MAX_NEXT_WEIGHT;

    #[test]
    fn test_posting_operations() {
        let mut builder = PostingBuilder::new();
        builder.add(1, 1.0);
        builder.add(2, 2.1);
        builder.add(5, 5.0);
        builder.add(3, 2.0);
        builder.add(8, 3.4);
        builder.add(10, 3.0);
        builder.add(20, 3.0);
        builder.add(7, 4.0);
        builder.add(11, 3.0);

        let posting_list = builder.build();

        let mut iter = PostingListIterator::new(&posting_list.elements);

        assert_eq!(iter.peek().unwrap().record_id, 1);
        iter.advance();
        assert_eq!(iter.peek().unwrap().record_id, 2);
        iter.advance();
        assert_eq!(iter.peek().unwrap().record_id, 3);

        assert_eq!(iter.skip_to(7).unwrap().record_id, 7);
        assert_eq!(iter.peek().unwrap().record_id, 7);

        assert!(iter.skip_to(9).is_none());
        assert_eq!(iter.peek().unwrap().record_id, 10);

        assert!(iter.skip_to(20).is_some());
        assert_eq!(iter.peek().unwrap().record_id, 20);

        assert!(iter.skip_to(21).is_none());
        assert!(iter.peek().is_none());
    }

    #[test]
    fn test_upsert_insert_last() {
        let mut builder = PostingBuilder::new();
        builder.add(1, 1.0);
        builder.add(3, 3.0);
        builder.add(2, 2.0);

        let mut posting_list = builder.build();

        // sorted by id
        assert_eq!(posting_list.elements[0].record_id, 1);
        assert_eq!(posting_list.elements[0].weight, 1.0);
        assert_eq!(posting_list.elements[0].max_next_weight, 3.0);

        assert_eq!(posting_list.elements[1].record_id, 2);
        assert_eq!(posting_list.elements[1].weight, 2.0);
        assert_eq!(posting_list.elements[1].max_next_weight, 3.0);

        assert_eq!(posting_list.elements[2].record_id, 3);
        assert_eq!(posting_list.elements[2].weight, 3.0);
        assert_eq!(
            posting_list.elements[2].max_next_weight,
            DEFAULT_MAX_NEXT_WEIGHT
        );

        // insert mew last element
        posting_list.upsert(PostingElementEx::new(4, 4.0));
        assert_eq!(posting_list.elements[3].record_id, 4);
        assert_eq!(posting_list.elements[3].weight, 4.0);
        assert_eq!(
            posting_list.elements[3].max_next_weight,
            DEFAULT_MAX_NEXT_WEIGHT
        );

        // must update max_next_weight of previous elements if necessary
        for element in posting_list.elements.iter().take(3) {
            assert_eq!(element.max_next_weight, 4.0);
        }
    }

    #[test]
    fn test_upsert_insert_in_gap() {
        let mut builder = PostingBuilder::new();
        builder.add(1, 1.0);
        builder.add(3, 3.0);
        builder.add(2, 2.0);
        // no entry for 4
        builder.add(5, 5.0);

        let mut posting_list = builder.build();

        // sorted by id
        assert_eq!(posting_list.elements[0].record_id, 1);
        assert_eq!(posting_list.elements[0].weight, 1.0);
        assert_eq!(posting_list.elements[0].max_next_weight, 5.0);

        assert_eq!(posting_list.elements[1].record_id, 2);
        assert_eq!(posting_list.elements[1].weight, 2.0);
        assert_eq!(posting_list.elements[1].max_next_weight, 5.0);

        assert_eq!(posting_list.elements[2].record_id, 3);
        assert_eq!(posting_list.elements[2].weight, 3.0);
        assert_eq!(posting_list.elements[2].max_next_weight, 5.0);

        assert_eq!(posting_list.elements[3].record_id, 5);
        assert_eq!(posting_list.elements[3].weight, 5.0);
        assert_eq!(
            posting_list.elements[3].max_next_weight,
            DEFAULT_MAX_NEXT_WEIGHT
        );

        // insert mew last element
        posting_list.upsert(PostingElementEx::new(4, 4.0));

        // `4` is shifted to the right
        assert_eq!(posting_list.elements[4].record_id, 5);
        assert_eq!(posting_list.elements[4].weight, 5.0);
        assert_eq!(
            posting_list.elements[4].max_next_weight,
            DEFAULT_MAX_NEXT_WEIGHT
        );

        // new element
        assert_eq!(posting_list.elements[3].record_id, 4);
        assert_eq!(posting_list.elements[3].weight, 4.0);

        // must update max_next_weight of previous elements
        for element in posting_list.elements.iter().take(4) {
            assert_eq!(element.max_next_weight, 5.0);
        }
    }

    #[test]
    fn test_upsert_update() {
        let mut builder = PostingBuilder::new();
        builder.add(1, 1.0);
        builder.add(3, 3.0);
        builder.add(2, 2.0);

        let mut posting_list = builder.build();

        // sorted by id
        assert_eq!(posting_list.elements[0].record_id, 1);
        assert_eq!(posting_list.elements[0].weight, 1.0);
        assert_eq!(posting_list.elements[0].max_next_weight, 3.0);

        assert_eq!(posting_list.elements[1].record_id, 2);
        assert_eq!(posting_list.elements[1].weight, 2.0);
        assert_eq!(posting_list.elements[1].max_next_weight, 3.0);

        assert_eq!(posting_list.elements[2].record_id, 3);
        assert_eq!(posting_list.elements[2].weight, 3.0);
        assert_eq!(
            posting_list.elements[2].max_next_weight,
            DEFAULT_MAX_NEXT_WEIGHT
        );

        // increase weight of existing element
        posting_list.upsert(PostingElementEx::new(2, 4.0));

        assert_eq!(posting_list.elements[0].record_id, 1);
        assert_eq!(posting_list.elements[0].weight, 1.0);
        assert_eq!(posting_list.elements[0].max_next_weight, 4.0); // update propagated

        assert_eq!(posting_list.elements[1].record_id, 2);
        assert_eq!(posting_list.elements[1].weight, 4.0); // updated
        assert_eq!(posting_list.elements[1].max_next_weight, 3.0);

        assert_eq!(posting_list.elements[2].record_id, 3);
        assert_eq!(posting_list.elements[2].weight, 3.0);
        assert_eq!(
            posting_list.elements[2].max_next_weight,
            DEFAULT_MAX_NEXT_WEIGHT
        );
    }

    /// After any sequence of inserts (in and out of id order), weight updates (up, down and
    /// unchanged) and deletes, each element's `max_next_weight` is bit for bit the one a fresh
    /// build of the same records computes: the maximum weight to its right.
    #[test]
    fn test_max_next_weight_after_random_operations() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for _ in 0..500 {
            let mut list = PostingList::default();
            let mut records = std::collections::BTreeMap::new();
            for step in 0..200_u32 {
                // Ascending appends first, then random ids: few distinct weights, so ties occur.
                let id = if step < 40 { step * 2 } else { next(96) as u32 };
                let weight = (next(12) + 1) as f32 / 4.0;
                if step >= 40 && next(4) == 0 {
                    list.delete(id);
                    records.remove(&id);
                } else {
                    list.upsert(PostingElementEx::new(id, weight));
                    records.insert(id, weight);
                }
                let mut builder = PostingBuilder::new();
                for (&id, &weight) in &records {
                    builder.add(id, weight);
                }
                let expected = builder.build();
                let bits = |list: &PostingList| {
                    list.elements
                        .iter()
                        .map(|e| (e.record_id, e.weight.to_bits(), e.max_next_weight.to_bits()))
                        .collect_vec()
                };
                assert_eq!(bits(&list), bits(&expected));
            }
        }
    }

    /// The propagation stops at the first unchanged entry. Each case plants a marker in the
    /// first element's `max_next_weight`: a propagation that walked the whole list (the old,
    /// quadratic one) would overwrite it.
    #[test]
    fn test_propagation_stops_at_unchanged_entry() {
        let marker = 99.0_f32;
        let planted = |records: Vec<(PointOffsetType, DimWeight)>| {
            let mut list = PostingList::from(records);
            list.elements[0].max_next_weight = marker;
            list
        };
        let first_max_next_weight = |list: &PostingList| list.elements[0].max_next_weight;

        // Append: the new last element raises only its predecessor.
        let mut list = planted(vec![(0, 1.0), (1, 5.0), (2, 2.0)]);
        list.upsert(PostingElementEx::new(3, 1.0));
        assert_eq!(first_max_next_weight(&list), marker);

        // Update: lowering the last weight changes only its predecessor.
        let mut list = planted(vec![(0, 1.0), (1, 5.0), (2, 2.0), (3, 1.0)]);
        list.upsert(PostingElementEx::new(3, 0.5));
        assert_eq!(first_max_next_weight(&list), marker);

        // Delete: removing the last element changes only the new last one's predecessor.
        let mut list = planted(vec![(0, 1.0), (1, 5.0), (2, 2.0), (3, 1.0), (4, 1.0)]);
        list.delete(4);
        assert_eq!(first_max_next_weight(&list), marker);
    }

    /// An element inserted in the middle starts with `DEFAULT_MAX_NEXT_WEIGHT`, which equals
    /// the propagated value when every weight to its right is negative infinity. It must still
    /// be written past, so its own weight reaches its predecessors.
    #[test]
    fn test_insert_before_negative_infinity_tail() {
        let mut list = PostingList::from(vec![(0, 1.0), (2, f32::NEG_INFINITY)]);
        list.upsert(PostingElementEx::new(1, 5.0));
        assert_eq!(list.elements[0].max_next_weight, 5.0);
    }

    #[test]
    fn test_random_delete() {
        use rand::RngExt;
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let mut ids = Vec::new();
            let mut cur_id = 0;
            for _ in 0..32 {
                cur_id += rng.random_range(1..10);
                ids.push(cur_id);
            }
            ids.shuffle(&mut rng);
            let random_id = ids[rng.random_range(0..ids.len())];

            let mut builder1 = PostingBuilder::new();
            let mut builder2 = PostingBuilder::new();
            for id in ids {
                let val = rng.random_range(0..100) as f32 / 10.0;
                builder1.add(id, val);
                if id != random_id {
                    builder2.add(id, val);
                }
            }

            let mut posting_list1 = builder1.build();
            posting_list1.delete(random_id);
            let posting_list2 = builder2.build();

            // Ok
            assert_eq!(
                posting_list1
                    .elements
                    .iter()
                    .map(|e| e.record_id)
                    .collect_vec(),
                posting_list2
                    .elements
                    .iter()
                    .map(|e| e.record_id)
                    .collect_vec(),
            );
            assert_eq!(
                posting_list1
                    .elements
                    .iter()
                    .map(|e| e.weight)
                    .collect_vec(),
                posting_list2
                    .elements
                    .iter()
                    .map(|e| e.weight)
                    .collect_vec(),
            );

            // Fail
            assert_eq!(
                posting_list1
                    .elements
                    .iter()
                    .map(|e| e.max_next_weight)
                    .collect_vec(),
                posting_list2
                    .elements
                    .iter()
                    .map(|e| e.max_next_weight)
                    .collect_vec(),
            );

            // Ok (at least they won't break pruning logic)
            assert!(
                std::iter::zip(&posting_list1.elements, &posting_list2.elements,)
                    .all(|(e1, e2)| e1.max_next_weight >= e2.max_next_weight),
            );
        }
    }
}

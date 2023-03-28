use itertools::{EitherOrBoth, Itertools};

use crate::try_iter::PairOrSingle;

use super::try_iter::{
    try_pair_with_slice, try_validate, IndexIsOutOfBounds, InvalidPairOrItem, PairValidator,
};

/// Plucks items from the supplied iterator corresponding to missed indices.
/// This function implies that both iterators are sorted.
pub fn pluck_missed<Indices, Iter>(indices: Indices, iter: Iter) -> impl Iterator<Item = Iter::Item>
where
    Indices: IntoIterator<Item = usize>,
    Iter: IntoIterator,
{
    iter.into_iter()
        .enumerate()
        .merge_join_by(indices, |(i, _), j| i.cmp(j))
        .filter_map(|either| match either {
            EitherOrBoth::Left((_, item)) => Some(item),
            _ => None,
        })
}

/// Maps supplied iterator and attempts to pair each item with an item from the slice which has provided index.
/// Returns `Err` containing an invalid index in case slice length is exceeded.
pub fn pair_with_slice<'iter, 'pairs, I, Item, P>(
    iter: I,
    pair_with: &'pairs [P],
) -> impl Iterator<Item = Result<(&'pairs P, Item), IndexIsOutOfBounds>> + 'iter
where
    'pairs: 'iter,
    I: IntoIterator<Item = (usize, Item)> + 'iter,
    Item: 'iter,
{
    try_pair_with_slice(iter.into_iter().map(Ok), pair_with)
}

/// Maps supplied iterator and attempts to pair each successfully validated item with a corresponding item from the slice.
/// Validation errors will be propagated without looking at them.
/// In case of error, `Err(IndexIsOutOfBounds)` will be emitted.
pub fn pair_valid_items_with_slice<'iter, 'pairs, I, Item, Pair, E, P>(
    iter: I,
    validator: P,
    pair_with: &'pairs [Pair],
) -> impl Iterator<Item = Result<(&'pairs Pair, Item), E>> + 'iter
where
    'pairs: 'iter,
    I: IntoIterator<Item = (usize, Item)> + 'iter,
    Item: 'iter,
    P: PairValidator<(usize, Item)> + 'iter,
    E: From<IndexIsOutOfBounds> + From<InvalidPairOrItem<P::ValidationItem>> + 'iter,
{
    try_pair_with_slice(try_validate(iter.into_iter().map(Ok), validator), pair_with)
}

/// Ensures that the given iterator satisfies provided function for each previous - current pair.
/// The supplied option will be modified to invalid pair in case of failure, and iteration will be aborted.
pub fn take_while_satisfy<'iter, 'invalid, I, P>(
    iter: I,
    validator: P,
    invalid: &'invalid mut Option<PairOrSingle<P::ValidationItem>>,
) -> impl Iterator<Item = I::Item> + 'iter
where
    'invalid: 'iter,
    I: IntoIterator + 'iter,
    P: PairValidator<I::Item> + 'iter,
{
    try_validate(iter.into_iter().map(Ok), validator)
        .map(|res| {
            res.map_err(|InvalidPairOrItem(pair_or_single)| invalid.replace(pair_or_single))
                .ok()
        })
        .take_while(Option::is_some)
        .flatten()
}

/// Skips up to `n` elements from the iterator using supplied random generator.
pub fn skip_up_to_n<'rng, I>(
    rng: &'rng mut impl ark_std::rand::RngCore,
    iter: I,
    mut allowed_to_skip: usize,
) -> impl Iterator<Item = I::Item> + 'rng
where
    I: IntoIterator + 'rng,
{
    iter.into_iter().filter(move |_| {
        use ark_std::rand::Rng;

        let res = allowed_to_skip == 0 || rng.gen_bool(0.5);
        if !res {
            allowed_to_skip -= 1;
        }

        res
    })
}

#[cfg(test)]
mod tests {
    use crate::misc::{check_seq_from, pair_is_lt};

    use super::*;

    #[test]
    fn valid_take_while_unique_sorted() {
        let mut opt = None;
        let values: Vec<_> = take_while_satisfy(1..10, pair_is_lt, &mut opt).collect();

        assert_eq!(values, (1..10).collect::<Vec<_>>());
        assert_eq!(opt, None);

        let values: Vec<_> = take_while_satisfy([2, 8, 9], pair_is_lt, &mut opt).collect();
        assert_eq!(values, [2, 8, 9]);
        assert_eq!(opt, None);
    }

    #[test]
    fn invalid_take_while_unique_sorted() {
        let mut opt = None;
        let values: Vec<_> =
            take_while_satisfy([5, 6, 7, 9, 10, 8], pair_is_lt, &mut opt).collect();

        assert_eq!(values, vec![5, 6, 7, 9, 10]);
        assert_eq!(opt, Some(PairOrSingle::Pair(10, 8)));

        let values: Vec<_> = take_while_satisfy([100, 0], pair_is_lt, &mut opt).collect();
        assert_eq!(values, [100]);
        assert_eq!(opt, Some(PairOrSingle::Pair(100, 0)));
    }

    #[test]
    fn sequence_from() {
        let mut invalid = None;
        assert_eq!(
            take_while_satisfy(0..5, check_seq_from(0), &mut invalid).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert_eq!(invalid, None);

        assert_eq!(
            take_while_satisfy(1..5, check_seq_from(0), &mut invalid).collect::<Vec<_>>(),
            vec![]
        );
        assert_eq!(invalid, Some(PairOrSingle::Single(1)));

        assert_eq!(
            take_while_satisfy(vec![1, 2, 3, 6, 7], check_seq_from(1), &mut invalid)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(invalid, Some(PairOrSingle::Pair(3, 6)));
    }

    #[test]
    fn check_pluck_missed() {
        assert_eq!(
            pluck_missed([1, 3], [0, 1, 2]).collect::<Vec<_>>(),
            vec![0, 2]
        );
        assert_eq!(
            pluck_missed([3, 5], 0..10).collect::<Vec<_>>(),
            [0, 1, 2, 4, 6, 7, 8, 9]
        );
    }
}

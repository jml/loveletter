// XXX: This is algorithmically slow and probably slow in other ways.
// I'm just doing the bare minimum, as I want to pop my yak-stack a little.
//
// NB: it's only order-preserving because it makes the tests easier.
pub fn subtract_vector<A: PartialEq>(xs: Vec<A>, ys: &[A]) -> Option<Vec<A>> {
    let mut zs = xs;
    for y in ys.iter() {
        let pos = zs.iter().position(|x| x == y);
        match pos {
            Some(i) => { zs.remove(i); }
            None => return None
        };
    }
    Some(zs)
}

/// Given a tuple `(a, b)` and a value `x`, return whichever of `(a, b)` is
/// not `x`. If neither is `x`, return None.
pub fn other<T: Eq>((a, b): (T, T), x: T) -> Option<T> {
    if x == a { Some(b) } else { if x == b { Some(a) } else { None } }
}


// TODO: Write tests.
// TODO: Figure out if this can be written in terms of slices rather than
// vectors.

/// Return the "biggest" elements of `xs`, where `f` is a function applied to
/// each element that returns some indication of magnitude.
pub fn maxima_by<'a, A, B: Ord, F>(xs: &'a Vec<A>, f: F) -> Vec<&'a A>
    where F: Fn(&A) -> B {
    let mut maxes = vec![];
    let indexes: Vec<(B, usize)> = xs.iter().enumerate().map(|(i, a)| (f(a), i)).collect();
    let max = indexes.iter().max_by(|&&(ref b, _)| b);
    match max {
        None => maxes,
        Some(&(ref value, _)) => {
            for &(ref b, i) in indexes.iter() {
                if *b >= *value {
                    maxes.push(&xs[i])
                }
            }
            maxes
        },
    }
}


#[cfg(test)]
mod test {
    use super::other;
    use super::subtract_vector;

    #[test]
    fn test_vector_diff_trivial() {
        let xs: Vec<i32> = vec![];
        let ys = [];
        assert_eq!(Some(vec![]), subtract_vector(xs, &ys))
    }

    #[test]
    fn test_vector_diff_identity() {
        let xs: Vec<i32> = vec![1, 2, 3];
        let ys = [];
        assert_eq!(Some(vec![1, 2, 3]), subtract_vector(xs, &ys))
    }

    #[test]
    fn test_vector_diff_removes() {
        let xs: Vec<i32> = vec![1, 2, 3];
        let ys = [2];
        assert_eq!(Some(vec![1, 3]), subtract_vector(xs, &ys))
    }

    #[test]
    fn test_vector_diff_only_removes_one() {
        let xs: Vec<i32> = vec![1, 2, 3, 2];
        let ys = [2];
        assert_eq!(Some(vec![1, 3, 2]), subtract_vector(xs, &ys))
    }

    #[test]
    fn test_vector_diff_contains_excess_elements() {
        let xs: Vec<i32> = vec![1, 2, 3, 2];
        let ys = [2, 2, 2];
        assert_eq!(None, subtract_vector(xs, &ys))
    }

    #[test]
    fn test_vector_diff_contains_novel_elements() {
        let xs: Vec<i32> = vec![1, 2, 3, 2];
        let ys = [4];
        assert_eq!(None, subtract_vector(xs, &ys))
    }

    #[test]
    fn test_other_one() {
        assert_eq!(None, other((1, 2), 0));
        assert_eq!(Some(2), other((1, 2), 1));
        assert_eq!(Some(1), other((1, 2), 2));
    }

    #[test]
    fn test_maxima_by_no_data() {
        let xs: Vec<i32> = vec![];
        let empty: Vec<&i32> = vec![];
        assert_eq!(empty, super::maxima_by(&xs, |&x| x));
    }

    #[test]
    fn test_maxima_by_id_function() {
        let xs: Vec<i32> = vec![1, 2, 3, 2, 3, 1];
        assert_eq!(vec![&xs[2], &xs[4]], super::maxima_by(&xs, |&x| x));
    }

    #[test]
    fn test_maxima_by_interesting_function() {
        let xs = vec!["hello", "cat", "dog", "world"];
        assert_eq!(vec![&xs[0], &xs[3]], super::maxima_by(&xs, |&x| x.len()));
    }

}

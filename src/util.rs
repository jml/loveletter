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

#[cfg(test)]
mod test {
    use super::other;
    use super::subtract_vector;

    #[test]
    fn test_vector_diff_trivial() {
        let xs: Vec<int> = vec![];
        let ys = [];
        assert_eq!(Some(vec![]), subtract_vector(xs, ys))
    }

    #[test]
    fn test_vector_diff_identity() {
        let xs: Vec<int> = vec![1, 2, 3];
        let ys = [];
        assert_eq!(Some(vec![1, 2, 3]), subtract_vector(xs, ys))
    }

    #[test]
    fn test_vector_diff_removes() {
        let xs: Vec<int> = vec![1, 2, 3];
        let ys = [2];
        assert_eq!(Some(vec![1, 3]), subtract_vector(xs, ys))
    }

    #[test]
    fn test_vector_diff_only_removes_one() {
        let xs: Vec<int> = vec![1, 2, 3, 2];
        let ys = [2];
        assert_eq!(Some(vec![1, 3, 2]), subtract_vector(xs, ys))
    }

    #[test]
    fn test_vector_diff_contains_excess_elements() {
        let xs: Vec<int> = vec![1, 2, 3, 2];
        let ys = [2, 2, 2];
        assert_eq!(None, subtract_vector(xs, ys))
    }

    #[test]
    fn test_vector_diff_contains_novel_elements() {
        let xs: Vec<int> = vec![1, 2, 3, 2];
        let ys = [4];
        assert_eq!(None, subtract_vector(xs, ys))
    }

    #[test]
    fn test_other_one() {
        assert_eq!(None, other((1u, 2u), 0u));
        assert_eq!(Some(2), other((1u, 2u), 1u));
        assert_eq!(Some(1), other((1u, 2u), 2u));
    }
}

// We want to make sure we are getting the shortest match possible
// without getting tripped up by pathological cases.
pub mod minspan {

    pub fn span<A>(query: &Vec<A>, history: &Vec<A>) -> Option<(usize, usize)>
    where
        A: PartialEq,
    {
        // If history is empty, we cannot find any span with valid indices.
        if history.is_empty() {
            return None;
        }
        // If query is empty, it's a subsequence starting at index 0.
        // The current implementation returns (0, 0), representing history[0..=0].
        // This is valid since history is non-empty here.
        if query.is_empty() {
            return Some((0, 0));
        }

        // Initialize state for the main algorithm
        let mut starting_at: Vec<Option<(usize, usize)>> = query.iter().map(|_| None).collect();
        let mut best_complete_solution: Option<(usize, usize)> = None;

        // Main loop: requires non-empty query and history
        for (bodyindex, bodychr) in history.iter().enumerate() {
            for (keyindex, keychr) in query.iter().enumerate().rev() {
                if keychr == bodychr {
                    // we have a match, therefore record it: it ends at bodyindex,
                    // and by construction, starts at starting_at[0]
                    starting_at[keyindex] = if keyindex == 0 {
                        // we got nothing yet! set to beginning
                        Some((bodyindex, bodyindex))
                    } else {
                        starting_at[keyindex - 1].map(|(start, _end)| (start, bodyindex))
                    };
                    // are we finished?
                    if (keyindex + 1) == query.len() {
                        if let Some((from, to)) = starting_at[keyindex] {
                            best_complete_solution = match best_complete_solution {
                                None => Some((from, to)), // 1+to - from),
                                Some((currfrom, currto)) => {
                                    Some(if to - from < currto - currfrom {
                                        (from, to)
                                    } else {
                                        (currfrom, currto)
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }
        best_complete_solution
    }
}

// Add proptest imports for property-based testing
#[cfg(test)]
extern crate proptest;
#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    // Keep existing tests
    #[test]
    fn test_minimal_match() {
        // Helper to convert string slices to Vec<char> and call span
        let run_span = |needle: &str, haystack: &str| {
            let query: Vec<char> = needle.chars().collect();
            let history: Vec<char> = haystack.chars().collect();
            minspan::span(&query, &history)
        };

        // Helper to get span length or None
        let get_span_len = |needle: &str, haystack: &str| {
            run_span(needle, haystack).map(|(from, to)| 1 + to - from)
        };

        assert_eq!(get_span_len("ab", "ab"), Some(2));
        assert_eq!(get_span_len("a", "ab"), Some(1));
        assert_eq!(get_span_len("ab", "abc"), Some(2));
        assert_eq!(get_span_len("abc", "abcd"), Some(3));
        assert_eq!(get_span_len("curl", "curly"), Some(4));
        assert_eq!(get_span_len("curl", "acccccurlycurrelly"), Some(4));
        assert_eq!(get_span_len("z", "acccccurlycurrelly"), None);
        assert_eq!(get_span_len("ssh", "testssh"), Some(3));
        assert_eq!(get_span_len("aba", "abababa"), Some(3));
        assert_eq!(run_span("", "abc"), Some((0, 0))); // Empty query
        assert_eq!(run_span("a", ""), None); // Empty history
        assert_eq!(run_span("", ""), None); // Both empty
    }

    #[test]
    fn test_is_subsequence_consumes_main_seq() {
        let sub: Vec<char> = vec!['a', 'a', 'a'];
        let main_seq: Vec<char> = vec!['a', 'a'];
        assert!(!is_subsequence(&sub, &main_seq), "is_subsequence should return false when main_seq is consumed before sub is fully matched");

        let sub_ok: Vec<char> = vec!['a', 'a'];
        let main_seq_ok: Vec<char> = vec!['a', 'a', 'a'];
        assert!(
            is_subsequence(&sub_ok, &main_seq_ok),
            "is_subsequence should return true when sub is shorter or equal and matches"
        );

        let sub_diff: Vec<char> = vec!['a', 'b'];
        let main_seq_diff: Vec<char> = vec!['a', 'a'];
        assert!(
            !is_subsequence(&sub_diff, &main_seq_diff),
            "is_subsequence should return false for non-matching elements"
        );
    }

    // Helper function to check if `sub` is a subsequence of `main_seq`
    fn is_subsequence<A: PartialEq>(sub: &[A], main_seq: &[A]) -> bool {
        let mut main_iter = main_seq.iter();
        sub.iter()
            .all(|sub_item| main_iter.any(|main_item| main_item == sub_item))
    }

    // Use proptest! with a custom config and closure syntax
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn prop_span_finds_valid_subsequence(
            // Query uses chars 'a'..'c' via regex strategy, length 0..10
            query in proptest::collection::vec("[a-c]", 0..10),
            // History uses chars 'a'..'d' via regex strategy, length 0..50
            history in proptest::collection::vec("[a-d]", 0..50)
        ) {
            if let Some((from, to)) = minspan::span(&query, &history) {
                // 1. Indices must be valid
                prop_assert!(from <= to, "from index {} > to index {}", from, to);
                prop_assert!(to < history.len(), "to index {} is out of bounds for history len {}", to, history.len());

                // 2. The slice history[from..=to] must contain query as a subsequence
                let history_slice = &history[from..=to];
                prop_assert!(
                    is_subsequence(&query, history_slice),
                    "query {:?} is not a subsequence of history slice {:?} (from={}, to={})",
                    query, history_slice, from, to
                );

                // 3. Check if a shorter span exists *before* the found one (minimality check part 1)
                // This is tricky to fully verify without re-implementing the logic.
                // We can check that no shorter valid subsequence exists ending at or before `to`.
                // A simpler check: ensure the first element of query matches history[from]
                // if query is not empty. This isn't always true for the *minimal* span,
                // e.g., query="ab", history="axb", span=(1,2) 'xb', not (0,2) 'axb'.

                // 3. Minimality Check: Ensure no shorter span contains the query.
                // Iterate through all possible start (f) and end (t) indices.
                for f in 0..history.len() {
                    for t in f..history.len() {
                        // Check only spans that are strictly shorter than the found span (to - from).
                        if t - f < to - from {
                            let shorter_slice = &history[f..=t];
                            prop_assert!(
                                !is_subsequence(&query, shorter_slice),
                                "Found shorter span history[{}:{}] ({:?}) that contains query {:?}, but span returned ({}, {}) len {}",
                                f, t, shorter_slice, query, from, to, to - from + 1
                            );
                        }
                    }
                }

            } else {
                // If span returns None, then query should not be a subsequence of history,
                // unless the query itself is empty (as empty query is always a subsequence).
                 prop_assert!(
                    !is_subsequence(&query, &history) || query.is_empty(),
                    "span returned None, but query {:?} IS a subsequence of history {:?}",
                    query, history
                );
            }
        }
    }
}

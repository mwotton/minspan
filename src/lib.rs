// We want to make sure we are getting the shortest match possible
// without getting tripped up by pathological cases.
pub mod minspan {

    pub fn span<A>(query: &Vec<A>, history: &Vec<A>) -> Option<(usize, usize)>
    where
        A: PartialEq,
    {
        let mut starting_at: Vec<Option<(usize, usize)>> = query.iter().map(|_| None).collect();
        let mut best_complete_solution: Option<(usize, usize)> = None;

        if query.len() == 0 {
            return Some((0, 0));
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_match() {
        let wrapper = |needle: &str, haystack: &str| match minspan::span(
            &needle.chars().collect(),
            &haystack.chars().collect(),
        ) {
            Some((from, to)) => Some(1 + to - from),
            None => None,
        };

        assert_eq!(wrapper("ab", "ab").unwrap(), 2);
        assert_eq!(wrapper("a", "ab").unwrap(), 1);
        assert_eq!(wrapper("ab", "abc").unwrap(), 2);
        assert_eq!(wrapper("abc", "abcd").unwrap(), 3);
        assert_eq!(wrapper("curl", "curly").unwrap(), 4);
        assert_eq!(wrapper("curl", "acccccurlycurrelly").unwrap(), 4);
        assert_eq!(wrapper("z", "acccccurlycurrelly"), None);
        assert_eq!(wrapper("ssh", "testssh"), Some(3));
        assert_eq!(wrapper("aba", "abababa"), Some(3));
    }
}

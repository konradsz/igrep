use grep::{
    matcher::Matcher,
    searcher::{Searcher, Sink, SinkMatch},
};

use super::grep_match::GrepMatch;

pub(crate) struct MatchesSink<'a, M>
where
    M: Matcher,
{
    matcher: M,
    matches_in_entry: &'a mut Vec<GrepMatch>,
}

impl<'a, M> MatchesSink<'a, M>
where
    M: Matcher,
{
    pub(crate) fn new(matcher: M, matches_in_entry: &'a mut Vec<GrepMatch>) -> Self {
        Self {
            matcher,
            matches_in_entry,
        }
    }
}

fn split_by_lines(line_number: u64, text: &str, offsets: Vec<(usize, usize)>) -> Vec<GrepMatch> {
    let mut matches = Vec::new();
    if !text.trim().contains('\n') {
        matches.push(GrepMatch::new(line_number, text.into(), offsets));
    } else {
        // handles multiline searches
        let Some(&(first_offset_start, _)) = offsets.first() else {
            return Vec::new();
        };
        let Some((_, mut last_offset_end)) = offsets.last() else {
            return Vec::new();
        };
        for (idx, el) in text.lines().enumerate() {
            let start = if idx == 0 { first_offset_start } else { 0 };
            matches.push(GrepMatch::new(
                line_number + idx as u64,
                el.to_string(),
                vec![(start, std::cmp::min(el.len(), last_offset_end))],
            ));
            last_offset_end = last_offset_end.saturating_sub(el.len() + 1);
        }
    }

    matches
}

impl<M> Sink for MatchesSink<'_, M>
where
    M: Matcher,
{
    type Error = std::io::Error;

    fn matched(&mut self, _: &Searcher, sink_match: &SinkMatch) -> Result<bool, std::io::Error> {
        let line_number = sink_match
            .line_number()
            .ok_or(std::io::ErrorKind::InvalidData)?;
        let text =
            std::str::from_utf8(sink_match.bytes()).map_err(|_| std::io::ErrorKind::InvalidData)?;

        let mut offsets = vec![];
        self.matcher
            .find_iter(sink_match.bytes(), |m| {
                offsets.push((m.start(), m.end()));
                true
            })
            .ok();

        *self.matches_in_entry = split_by_lines(line_number, text, offsets);

        Ok(true)
    }
}

// TESTS:

// in file2 does searching for 2 empty lines should work?
// like `rg -U "file\n\nfile"

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_single_match() {
        let text = "lorem ipsum dolor sit amet\n";
        let matches = split_by_lines(0, text, vec![(5, 9)]);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, text);
        assert_eq!(matches[0].match_offsets, &[(5, 9)]);
    }

    #[test]
    fn single_line_multiple_matches() {
        let text = "lorem ipsum dolor sit amet\n";
        let matches = split_by_lines(0, text, vec![(0, 4), (8, 12)]);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, text);
        assert_eq!(matches[0].match_offsets, &[(0, 4), (8, 12)]);
    }

    #[test]
    fn multi_line() {
        let text = "lorem\nipsum dolor\nsit amet\n";
        let matches = split_by_lines(0, text, vec![(0, 21)]);

        assert_eq!(matches.len(), 3);
        let (m1, m2, m3) = (&matches[0], &matches[1], &matches[2]);
        assert_eq!(m1.line_number, 0);
        assert_eq!(m1.match_offsets, &[(0, 5)]);
        assert_eq!(m1.text, "lorem");

        assert_eq!(m2.line_number, 1);
        assert_eq!(m2.match_offsets, &[(0, 11)]);
        assert_eq!(m2.text, "ipsum dolor");

        assert_eq!(m3.line_number, 2);
        assert_eq!(m3.match_offsets, &[(0, 3)]);
        assert_eq!(m3.text, "sit amet");
    }
}

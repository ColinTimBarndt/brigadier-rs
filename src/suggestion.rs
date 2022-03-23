use std::{borrow::Cow, collections::HashSet, future::Future, ops::Range, pin::Pin};

use crate::{
    async_fn_type, context::CommandContext, context::StringRange, errors::CommandSyntaxError,
};

/// `'t`: Lifetime of borrowed suggestions text\
/// `'m`: Lifetime of borrowed tooltips
pub type SuggestionProvider<'i, 't, 'm, S> = async_fn_type!((CommandContext<S>, SuggestionsBuilder<'i, 't, 'm>) -> Result<Suggestions<'t, 'm>, CommandSyntaxError<'i>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestions<'t, 'm> {
    range: StringRange,
    suggestions: Vec<Suggestion<'t, 'm>>,
}

impl Suggestions<'static, 'static> {
    pub const EMPTY: Self = Suggestions::new(0..0, Vec::new());
}

impl<'t, 'm> Suggestions<'t, 'm> {
    pub const fn new(range: StringRange, suggestions: Vec<Suggestion<'t, 'm>>) -> Self {
        Self { range, suggestions }
    }
    /// Creates deduplicated suggestions expanded into the command.
    pub fn create(command: &str, suggestions: Vec<Suggestion<'t, 'm>>) -> Self {
        if suggestions.is_empty() {
            return Suggestions::EMPTY;
        }
        let mut start = usize::MAX;
        let mut end = usize::MIN;
        for suggestion in &suggestions {
            start = start.min(suggestion.range.start);
            end = end.max(suggestion.range.end);
        }
        let range = start..end;
        let mut texts = HashSet::with_capacity(suggestions.len());
        for suggestion in suggestions {
            texts.insert(suggestion.expand_owned(command, range.clone()));
        }
        let mut sorted: Vec<_> = texts.into_iter().collect();
        sorted.sort_by(Suggestion::cmp_ignore_case);
        Self::new(range, sorted)
    }
    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Suggestion<'t, 'm> {
    range: StringRange,
    text: Cow<'t, str>,
    int: Option<i32>,
    pub tooltip: Option<Cow<'m, str>>,
}
impl std::cmp::PartialOrd for Suggestion<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for Suggestion<'_, '_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.int, &other.int) {
            (Some(a), Some(b)) => a.cmp(b),
            _ => self.text.cmp(&other.text),
        }
    }
}

impl Suggestion<'_, '_> {
    pub fn range(&self) -> StringRange {
        self.range.clone()
    }
    pub fn int(&self) -> Option<i32> {
        self.int
    }
    // TODO: Could be optimized
    pub fn cmp_ignore_case(&self, other: &Self) -> std::cmp::Ordering {
        self.text.to_lowercase().cmp(&other.text.to_lowercase())
    }
}

impl<'t, 'm> Suggestion<'t, 'm> {
    pub fn text(&'t self) -> &'t str {
        &self.text
    }
    pub fn expand<'s>(&'s self, command: &str, range: StringRange) -> Cow<'s, Self> {
        if range == self.range {
            return Cow::Borrowed(self);
        }
        let Range {
            start: self_start,
            end: self_end,
        } = self.range;
        let mut result = String::with_capacity(
            self_start.saturating_sub(range.start)
                + self.text.len()
                + range.end.saturating_sub(self_end),
        );
        if range.start < self_start {
            result.push_str(&command[range.start..self_start]);
        }
        result.push_str(&self.text);
        if range.end > self_end {
            result.push_str(&command[self_end..range.end]);
        }
        Cow::Owned(Self {
            range,
            text: result.into(),
            tooltip: self.tooltip.clone(),
            ..Default::default()
        })
    }
    pub fn expand_owned<'s>(self, command: &str, range: StringRange) -> Self {
        if range == self.range {
            return self;
        }
        let Range {
            start: self_start,
            end: self_end,
        } = self.range;
        let mut result = String::with_capacity(
            self_start.saturating_sub(range.start)
                + self.text.len()
                + range.end.saturating_sub(self_end),
        );
        if range.start < self_start {
            result.push_str(&command[range.start..self_start]);
        }
        result.push_str(&self.text);
        if range.end > self_end {
            result.push_str(&command[self_end..range.end]);
        }
        Self {
            range,
            text: result.into(),
            tooltip: self.tooltip.clone(),
            ..Default::default()
        }
    }
    pub fn new_text(range: StringRange, text: impl Into<Cow<'t, str>>) -> Self {
        Self {
            range,
            text: text.into(),
            ..Default::default()
        }
    }
    pub fn new_text_with_tooltip(
        range: StringRange,
        text: impl Into<Cow<'t, str>>,
        tooltip: impl Into<Cow<'m, str>>,
    ) -> Self {
        Self {
            range,
            text: text.into(),
            tooltip: Some(tooltip.into()),
            ..Default::default()
        }
    }
    pub fn new_int(range: StringRange, int: i32) -> Self {
        Self {
            range,
            text: int.to_string().into(),
            int: Some(int),
            ..Default::default()
        }
    }
    pub fn new_int_with_tooltip(
        range: StringRange,
        int: i32,
        tooltip: impl Into<Cow<'m, str>>,
    ) -> Self {
        Self {
            range,
            text: int.to_string().into(),
            int: Some(int),
            tooltip: Some(tooltip.into()),
        }
    }
    /// Applies this suggestion to a string, "patching" the suggestion into it.
    pub fn apply(&'t self, input: &str) -> Cow<'t, str> {
        let Range {
            start: range_start,
            end: range_end,
        } = self.range;
        let input_len = input.len();
        if range_start == 0 && range_end == input_len {
            return (&self.text[..]).into();
        }
        let text_len = self.text.len();
        let mut result =
            String::with_capacity(range_start + text_len + input_len.saturating_sub(range_end));
        if range_start > 0 {
            result.push_str(&input[..range_start]);
        }
        result.push_str(&self.text);
        if range_end < input_len {
            result.push_str(&input[range_end..])
        }
        result.into()
    }
}

pub struct SuggestionsBuilder<'i, 't, 'm> {
    start: usize,
    input: &'i str,
    input_lower_case: &'i str,
    remaining: &'i str,
    remaining_lower_case: &'i str,
    result: Vec<Suggestion<'t, 'm>>,
}

impl<'i> SuggestionsBuilder<'i, '_, '_> {
    #[inline]
    pub fn input(&self) -> &'i str {
        self.input
    }
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn remaining(&self) -> &'i str {
        self.remaining
    }
    #[inline]
    pub fn remaining_lower_case(&self) -> &'i str {
        self.remaining_lower_case
    }
}

impl<'i, 't, 'm> SuggestionsBuilder<'i, 't, 'm> {
    #[inline]
    pub fn new(input: &'i str, input_lower_case: &'i str, start: usize) -> Self {
        Self {
            start,
            input,
            input_lower_case,
            remaining: &input[start..],
            remaining_lower_case: &input_lower_case[start..],
            result: Vec::new(),
        }
    }
    pub fn build(self) -> Suggestions<'t, 'm> {
        Suggestions::create(self.input, self.result)
    }
    pub fn suggest_text(&mut self, text: impl Into<Cow<'t, str>>) -> &mut Self {
        let text: Cow<'t, str> = text.into();
        if text == self.remaining {
            self
        } else {
            self.result
                .push(Suggestion::new_text(self.start..self.input.len(), text));
            self
        }
    }
    pub fn suggest_text_with_tooltip(
        &mut self,
        text: impl Into<Cow<'t, str>>,
        tooltip: impl Into<Cow<'m, str>>,
    ) -> &mut Self {
        let text: Cow<'t, str> = text.into();
        if text == self.remaining {
            self
        } else {
            self.result.push(Suggestion::new_text_with_tooltip(
                self.start..self.input.len(),
                text,
                tooltip.into(),
            ));
            self
        }
    }
    pub fn suggest_int(&mut self, int: i32) -> &mut Self {
        self.result
            .push(Suggestion::new_int(self.start..self.input.len(), int));
        self
    }
    pub fn suggest_int_with_tooltip(
        &mut self,
        int: i32,
        tooltip: impl Into<Cow<'m, str>>,
    ) -> &mut Self {
        self.result.push(Suggestion::new_int_with_tooltip(
            self.start..self.input.len(),
            int,
            tooltip.into(),
        ));
        self
    }
    pub fn add(mut self, other: &Self) -> Self {
        self.result.extend_from_slice(&other.result[..]);
        self
    }
    pub fn create_offset(&self, start: usize) -> Self {
        Self::new(self.input, self.input_lower_case, start)
    }
    pub fn restart(&self) -> Self {
        self.create_offset(self.start)
    }
}

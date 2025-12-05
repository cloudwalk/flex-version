#![doc = include_str!("../README.md")]

#[cfg(feature = "serde")]
mod serde;
#[cfg(test)]
mod tests;
mod util;

use std::{
    cmp::Ordering,
    error::Error,
    fmt::{self, Debug, Display},
    str::FromStr,
};

use util::SplitPrefix;

/// The characters that are hard delimiters for components.
/// This constant is public more as a matter of documentation than of utility.
pub const COMPONENT_SEPARATORS: &str = ".-_+:";

/// A component is a indivisible part of a version.
/// May be a number, or a alphabetic identifier.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Component {
    Identifier(Box<str>),
    Number(u64),
}

/// The default Component is the number zero.
impl Default for Component {
    fn default() -> Self {
        Self::Number(0)
    }
}

impl Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Component::Number(number) => write!(f, "{}", number),
            Component::Identifier(id) => f.write_str(id),
        }
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

/// A parser for components.
/// This is an iterator which will parse many components in succession.
#[derive(Debug)]
struct ComponentParser<'a> {
    /// Flag to indicate whether we're parsing the first component.
    first: bool,
    /// The input yet to be parsed.
    input: &'a str,
}

impl<'a> From<&'a str> for ComponentParser<'a> {
    fn from(input: &'a str) -> Self {
        Self { first: true, input }
    }
}

impl<'a> Iterator for ComponentParser<'a> {
    type Item = Result<Component, ParseVersionError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        if self.first {
            self.first = false;
        } else if let Some(tail) = self // Only allow separators after the first component.
            .input
            .strip_prefix(|c| COMPONENT_SEPARATORS.contains(c))
        {
            self.input = tail;
        }
        // Some versions have the format "7.4.1 (4452929)", so we must be able to
        // parse the trailing parenthesized string.
        else if let Some(tail) = self.input.strip_prefix(" (") {
            return if tail.trim_start_matches(|c| c != ')') == ")" {
                self.input = "";
                None
            } else {
                Some(Err(self.error()))
            };
        }

        // Try to parse a number.
        if let Some(component) = self.parse_number() {
            return Some(Ok(component));
        }

        // Try to parse an identifier.
        if let Some(component) = self.parse_identifier() {
            return Some(Ok(component));
        }

        Some(Err(self.error()))
    }
}

impl<'a> ComponentParser<'a> {
    /// Try to parse an integer of the given type.
    fn parse_integer<N: FromStr>(&mut self) -> Option<N> {
        if let Some((integer, tail)) = self.input.split_prefix(|c| c.is_ascii_digit()) {
            if let Ok(integer) = integer.parse() {
                self.input = tail;
                return Some(integer);
            }
        }

        None
    }

    /// Try to parse a number component.
    fn parse_number(&mut self) -> Option<Component> {
        self.parse_integer().map(Component::Number)
    }

    /// Try to parse an identifier component.
    fn parse_identifier(&mut self) -> Option<Component> {
        if let Some((identifier, tail)) = self.input.split_prefix(|c| c.is_ascii_alphabetic()) {
            self.input = tail;
            return Some(Component::Identifier(identifier.into()));
        }

        None
    }

    /// Generate a error with the current input.
    fn error(&self) -> ParseVersionError {
        ParseVersionError(self.input.to_owned())
    }
}

/// An error while parsing a version.
#[derive(Debug, Clone)]
pub struct ParseVersionError(String);

impl Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid version: {}", self.0)
    }
}

impl Error for ParseVersionError {}

/// A version. Versions are composed of one or more components, and provide a total
/// ordering.
#[derive(Clone)]
pub struct Version(Box<[Component]>);

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        let mut self_iter = self.0.iter().fuse();
        let mut other_iter = other.0.iter().fuse();

        loop {
            match (self_iter.next(), other_iter.next()) {
                (None, None) => return true,

                (None, Some(Component::Number(0))) => continue,
                (Some(Component::Number(0)), None) => continue,

                (None, Some(_)) => return false,
                (Some(_), None) => return false,

                (Some(c1), Some(c2)) if c1 == c2 => continue,
                (Some(_), Some(_)) => return false,
            }
        }
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut self_iter = self.0.iter().fuse();
        let mut other_iter = other.0.iter().fuse();

        loop {
            match (self_iter.next(), other_iter.next()) {
                (None, None) => return Ordering::Equal,

                (None, Some(Component::Number(0))) => continue,
                (Some(Component::Number(0)), None) => continue,

                (None, Some(Component::Number(_))) => return Ordering::Less,
                (Some(Component::Number(_)), None) => return Ordering::Greater,

                (None, Some(Component::Identifier(_))) => return Ordering::Greater,
                (Some(Component::Identifier(_)), None) => return Ordering::Less,

                (Some(c1), Some(c2)) if c1.cmp(c2) == Ordering::Equal => continue,
                (Some(c1), Some(c2)) => return c1.cmp(c2),
            }
        }
    }
}

/// The default version is `0.0.0`.
impl Default for Version {
    fn default() -> Self {
        Self(vec![Component::default(); 3].into_boxed_slice())
    }
}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ComponentParser::from(input)
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iterator = self.0.iter();

        if let Some(first) = iterator.next() {
            write!(f, "{}", first)?;
        }

        for component in iterator {
            write!(f, ".{}", component)?;
        }

        Ok(())
    }
}

impl Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

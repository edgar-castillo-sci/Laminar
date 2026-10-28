//! Core types: JamChain, Token, JamOption, Cell, Stabilizer.

use std::fmt;
use std::sync::Arc;

/// A single token in a JAM row.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Empty,
    Up(String),
    Down(String),
    Fixed(String),
    Buckled { up: String, down: String },
}

impl Token {
    /// Serialize to the JAM string form (e.g., `+j`, `+0`, `(+k-k)`).
    ///
    /// **Display only.** Comparison uses `Token::Ord` (`rank` below).
    pub fn to_jam(&self) -> String {
        match self {
            Token::Empty => "+0".to_string(),
            Token::Up(x) => format!("+{}", x),
            Token::Down(x) => format!("-{}", x),
            Token::Fixed(x) => format!("*{}", x),
            Token::Buckled { up, down } => format!("(+{}-{})", up, down),
        }
    }

    /// Canonical rank, per Definition 6 of *Orbits as Multisets*:
    /// Fixed (0) < Buckled (1) < Empty (2) < Up (3) < Down (4).
    /// Letters compared lexicographically within each variant.
    pub(crate) fn rank(&self) -> (u8, &str, &str) {
        match self {
            Token::Fixed(x) => (0, x.as_str(), ""),
            Token::Buckled { up, down } => (1, up.as_str(), down.as_str()),
            Token::Empty => (2, "", ""),
            Token::Up(x) => (3, x.as_str(), ""),
            Token::Down(x) => (4, x.as_str(), ""),
        }
    }
}

impl PartialOrd for Token {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Token {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

/// One of the eight canonical JAM options.
///
/// Named `JamOption` (not `Option`) to avoid shadowing
/// `std::option::Option` from the prelude.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JamOption {
    PL,
    BPL,
    BU,
    BBU,
    OneH,
    OneT,
    Janus1H,
    Janus1T,
}

impl JamOption {
    pub fn is_buckled(&self) -> bool {
        matches!(self, JamOption::BU | JamOption::BBU | JamOption::OneT | JamOption::Janus1T)
    }

    pub fn v_is_planar(&self) -> bool {
        matches!(self, JamOption::PL | JamOption::BPL | JamOption::OneH)
    }
}

/// The four combinatorial cells of *Stacking as Words*.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    PP,
    Pp,
    pP,
    Ap,
}

/// Stabilizer of a chain under G. |G| = 12, so |Stab| divides 12.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stabilizer {
    Trivial,
    Z2,
    Z3,
    Other(smallvec::SmallVec<[crate::GroupElement; 4]>),
}

impl Stabilizer {
    pub fn size(&self) -> usize {
        match self {
            Stabilizer::Trivial => 1,
            Stabilizer::Z2 => 2,
            Stabilizer::Z3 => 3,
            Stabilizer::Other(gens) => {
                let _ = gens;
                4
            }
        }
    }
}

/// A JAM chain: k rows × 3 columns, plus its option.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JamChain {
    pub(crate) rows: Arc<[[Token; 3]]>,
    pub(crate) option: JamOption,
}

impl JamChain {
    pub fn parse(s: &str, option: JamOption) -> Result<Self, ParseError> {
        let row_strs: Vec<&str> = s.split('/').collect();
        if row_strs.is_empty() {
            return Err(ParseError::Empty);
        }
        let mut rows = Vec::with_capacity(row_strs.len());
        for (i, row_str) in row_strs.iter().enumerate() {
            let row = parse_row(row_str)
                .map_err(|e| ParseError::Row { row: i, source: Box::new(e) })?;
            validate_row(&row, option, i)?;
            rows.push(row);
        }
        Ok(JamChain {
            rows: Arc::from(rows.into_boxed_slice()),
            option,
        })
    }

    pub fn option(&self) -> JamOption {
        self.option
    }

    pub fn k(&self) -> usize {
        self.rows.len()
    }

    /// Row-major serialization. **Display only**, not for comparison.
    pub fn raw(&self) -> String {
        self.rows
            .iter()
            .map(|row| row.iter().map(|t| t.to_jam()).collect::<String>())
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn rows(&self) -> &[[Token; 3]] {
        &self.rows
    }
}

impl fmt::Display for JamChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw())
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_row(s: &str) -> Result<[Token; 3], ParseError> {
    let mut tokens = Vec::with_capacity(3);
    let mut pos = 0;
    while pos < s.len() {
        let (token, consumed) = parse_token(&s[pos..])?;
        tokens.push(token);
        pos += consumed;
    }
    if tokens.len() != 3 {
        return Err(ParseError::InvalidRowLength(tokens.len()));
    }
    Ok([tokens[0].clone(), tokens[1].clone(), tokens[2].clone()])
}

fn parse_token(s: &str) -> Result<(Token, usize), ParseError> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Err(ParseError::UnexpectedEnd);
    }
    match bytes[0] {
        b'+' if bytes.len() >= 2 && bytes[1] == b'0' => Ok((Token::Empty, 2)),
        b'+' => {
            let c = s[1..].chars().next().ok_or(ParseError::UnexpectedEnd)?;
            Ok((Token::Up(c.to_string()), 1 + c.len_utf8()))
        }
        b'-' => {
            let c = s[1..].chars().next().ok_or(ParseError::UnexpectedEnd)?;
            Ok((Token::Down(c.to_string()), 1 + c.len_utf8()))
        }
        b'*' => {
            let c = s[1..].chars().next().ok_or(ParseError::UnexpectedEnd)?;
            Ok((Token::Fixed(c.to_string()), 1 + c.len_utf8()))
        }
        b'(' => {
            let end = s.find(')').ok_or(ParseError::UnclosedParen)?;
            let inner = &s[1..end];
            let (up, down) = parse_buckled(inner)?;
            Ok((Token::Buckled { up, down }, end + 1))
        }
        other => Err(ParseError::UnexpectedChar(other as char)),
    }
}

fn parse_buckled(inner: &str) -> Result<(String, String), ParseError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for c in inner.chars() {
        if (c == '+' || c == '-') && !current.is_empty() {
            parts.push(current.clone());
            current.clear();
        }
        current.push(c);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    if parts.len() != 2 {
        return Err(ParseError::InvalidBuckled(inner.to_string()));
    }
    let up = parts[0].trim_start_matches('+').to_string();
    let down = parts[1].trim_start_matches('-').to_string();
    Ok((up, down))
}

fn validate_row(row: &[Token; 3], option: JamOption, idx: usize) -> Result<(), ParseError> {
    match option {
        JamOption::OneH => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let ups = row.iter().filter(|t| matches!(t, Token::Up(_))).count();
            let buckleds = row
                .iter()
                .filter(|t| matches!(t, Token::Buckled { .. }))
                .count();
            if empties != 1 || ups != 1 || buckleds != 1 {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 1×Up, 1×Buckled",
                    got: (empties, ups, buckleds),
                });
            }
            for t in row {
                if let Token::Buckled { up, down } = t {
                    if up != down {
                        return Err(ParseError::JanusTokenInOneH {
                            row: idx,
                            up: up.clone(),
                            down: down.clone(),
                        });
                    }
                }
            }
            Ok(())
        }
        other => Err(ParseError::UnsupportedOption(other)),
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParseError {
    Empty,
    UnexpectedEnd,
    UnexpectedChar(char),
    UnclosedParen,
    InvalidBuckled(String),
    InvalidRowLength(usize),
    Row { row: usize, source: Box<ParseError> },
    InvalidRowForOption { row: usize, option: JamOption, expected: &'static str, got: (usize, usize, usize) },
    JanusTokenInOneH { row: usize, up: String, down: String },
    UnsupportedOption(JamOption),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty JAM string"),
            ParseError::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseError::UnexpectedChar(c) => write!(f, "unexpected character: '{}'", c),
            ParseError::UnclosedParen => write!(f, "unclosed parenthesis in buckled token"),
            ParseError::InvalidBuckled(s) => write!(f, "invalid buckled token: ({})", s),
            ParseError::InvalidRowLength(n) => write!(f, "row has {} tokens, expected 3", n),
            ParseError::Row { row, source } => write!(f, "row {}: {}", row, source),
            ParseError::InvalidRowForOption { row, option, expected, got } => write!(
                f,
                "row {}: invalid for option {:?} (expected {}, got {:?})",
                row, option, expected, got
            ),
            ParseError::JanusTokenInOneH { row, up, down } => write!(
                f,
                "row {}: Janus token (+{}-{}) not allowed in option 1H",
                row, up, down
            ),
            ParseError::UnsupportedOption(o) => {
                write!(f, "option {:?} not yet implemented (Fase 1)", o)
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mos2_bilayer() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(chain.k(), 2);
        assert_eq!(chain.raw(), "+j+0(+k-k)/+j+0(+k-k)");
    }

    #[test]
    fn parse_rejects_janus_token_in_oneh() {
        let err = JamChain::parse("+j+0(+k-l)/+j+0(+k-l)", JamOption::OneH).unwrap_err();
        assert!(matches!(err, ParseError::Row { .. } | ParseError::JanusTokenInOneH { .. }));
    }

    #[test]
    fn parse_rejects_wrong_row_length() {
        let err = JamChain::parse("+j+0/+j+0", JamOption::OneH).unwrap_err();
        assert!(matches!(err, ParseError::Row { .. }));
    }

    #[test]
    fn token_order_matches_paper() {
        // Paper: Fixed ≺ Buckled ≺ Empty ≺ Up ≺ Down
        assert!(Token::Fixed("j".into()) < Token::Buckled { up: "k".into(), down: "k".into() });
        assert!(Token::Buckled { up: "k".into(), down: "k".into() } < Token::Empty);
        assert!(Token::Empty < Token::Up("j".into()));
        assert!(Token::Up("j".into()) < Token::Down("j".into()));
    }
}

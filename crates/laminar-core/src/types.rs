//! Core types: JamChain, Token, JamOption, Cell, Stabilizer.

use std::fmt;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// JamOption
// ---------------------------------------------------------------------------

/// One of the eight canonical JAM options.
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
        matches!(
            self,
            JamOption::BU | JamOption::BBU | JamOption::OneT | JamOption::Janus1T
        )
    }

    pub fn v_is_planar(&self) -> bool {
        matches!(self, JamOption::PL | JamOption::BPL | JamOption::OneH)
    }
}

// ---------------------------------------------------------------------------
// Cell
// ---------------------------------------------------------------------------

/// The four combinatorial cells of *Stacking as Words*.
///
/// Mixed case is intentional: uppercase = "yes", lowercase = "no".
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    PP,
    Pp,
    pP,
    Ap,
}

// ---------------------------------------------------------------------------
// Stabilizer (with P1 fix)
// ---------------------------------------------------------------------------

/// Stabilizer of a chain under G. |G| = 12, so |Stab| divides 12.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stabilizer {
    Trivial,
    Z2,
    Z3,
    Other(smallvec::SmallVec<[crate::GroupElement; 4]>),
}

impl Stabilizer {
    /// Size of the stabilizer.
    ///
    /// **P1 fix (Fase 6.1):** for `Other(gens)`, the size is `1 + gens.len()`
    /// (identity + non-identity elements). Previously returned a
    /// placeholder `4`, which was wrong for |Stab| ∈ {6, 12}.
    pub fn size(&self) -> usize {
        match self {
            Stabilizer::Trivial => 1,
            Stabilizer::Z2 => 2,
            Stabilizer::Z3 => 3,
            Stabilizer::Other(gens) => 1 + gens.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// JamChain
// ---------------------------------------------------------------------------

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
            let row =
                parse_row(row_str).map_err(|e| ParseError::Row { row: i, source: Box::new(e) })?;
            validate_row(&row, option, i)?;
            rows.push(row);
        }
        Ok(JamChain {
            rows: Arc::from(rows.into_boxed_slice()),
            option,
        })
    }

    pub fn from_rows(rows: Vec<[Token; 3]>, option: JamOption) -> Self {
        Self {
            rows: Arc::from(rows.into_boxed_slice()),
            option,
        }
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

    /// The three columns as k-tuples of tokens (column-major).
    pub fn cols(&self) -> [Vec<Token>; 3] {
        let k = self.k();
        let mut cols: [Vec<Token>; 3] = [
            Vec::with_capacity(k),
            Vec::with_capacity(k),
            Vec::with_capacity(k),
        ];
        for row in self.rows() {
            cols[0].push(row[0].clone());
            cols[1].push(row[1].clone());
            cols[2].push(row[2].clone());
        }
        cols
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

// ---------------------------------------------------------------------------
// Row validation (all 8 options)
// ---------------------------------------------------------------------------

fn validate_row(row: &[Token; 3], option: JamOption, idx: usize) -> Result<(), ParseError> {
    match option {
        // PL (1): 1×Empty + 2×Up(same letter).
        JamOption::PL => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let ups: Vec<&String> = row
                .iter()
                .filter_map(|t| match t {
                    Token::Up(x) => Some(x),
                    _ => None,
                })
                .collect();
            if empties != 1 || ups.len() != 2 {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 2×Up(same letter)",
                    got: describe_row(row),
                });
            }
            if ups[0] != ups[1] {
                return Err(ParseError::MismatchedLetters {
                    row: idx,
                    option,
                    first: ups[0].clone(),
                    second: ups[1].clone(),
                });
            }
            Ok(())
        }
        // BPL (2): 1×Empty + 2×Up(different letters).
        JamOption::BPL => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let ups: Vec<&String> = row
                .iter()
                .filter_map(|t| match t {
                    Token::Up(x) => Some(x),
                    _ => None,
                })
                .collect();
            if empties != 1 || ups.len() != 2 {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 2×Up(different letters)",
                    got: describe_row(row),
                });
            }
            if ups[0] == ups[1] {
                return Err(ParseError::MismatchedLetters {
                    row: idx,
                    option,
                    first: ups[0].clone(),
                    second: ups[1].clone(),
                });
            }
            Ok(())
        }
        // BU (3): 1×Empty + 1×Up + 1×Down(same letter).
        JamOption::BU => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let up = row.iter().find_map(|t| match t {
                Token::Up(x) => Some(x),
                _ => None,
            });
            let down = row.iter().find_map(|t| match t {
                Token::Down(x) => Some(x),
                _ => None,
            });
            match (empties, up, down) {
                (1, Some(u), Some(d)) if u == d => Ok(()),
                (1, Some(u), Some(d)) => Err(ParseError::MismatchedLetters {
                    row: idx,
                    option,
                    first: u.clone(),
                    second: d.clone(),
                }),
                _ => Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 1×Up, 1×Down(same letter)",
                    got: describe_row(row),
                }),
            }
        }
        // BBU (4): 1×Empty + 1×Up + 1×Down(different letters).
        JamOption::BBU => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let up = row.iter().find_map(|t| match t {
                Token::Up(x) => Some(x),
                _ => None,
            });
            let down = row.iter().find_map(|t| match t {
                Token::Down(x) => Some(x),
                _ => None,
            });
            match (empties, up, down) {
                (1, Some(u), Some(d)) if u != d => Ok(()),
                (1, Some(u), Some(d)) => Err(ParseError::MismatchedLetters {
                    row: idx,
                    option,
                    first: u.clone(),
                    second: d.clone(),
                }),
                _ => Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 1×Up, 1×Down(different letters)",
                    got: describe_row(row),
                }),
            }
        }
        // OneH (5): 1×Empty + 1×Fixed(metal) + 1×Buckled(up==down).
        JamOption::OneH => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let fixeds = row.iter().filter(|t| matches!(t, Token::Fixed(_))).count();
            let buckleds = row
                .iter()
                .filter(|t| matches!(t, Token::Buckled { .. }))
                .count();
            if empties != 1 || fixeds != 1 || buckleds != 1 {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 1×Fixed, 1×Buckled",
                    got: describe_row(row),
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
        // OneT (6): 0×Empty + 1×Fixed(metal) + 1×Up + 1×Down(same letter).
        JamOption::OneT => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let fixeds = row.iter().filter(|t| matches!(t, Token::Fixed(_))).count();
            let up = row.iter().find_map(|t| match t {
                Token::Up(x) => Some(x),
                _ => None,
            });
            let down = row.iter().find_map(|t| match t {
                Token::Down(x) => Some(x),
                _ => None,
            });
            if empties != 0 || fixeds != 1 || up.is_none() || down.is_none() {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "0×Empty, 1×Fixed, 1×Up, 1×Down(same letter)",
                    got: describe_row(row),
                });
            }
            if up != down {
                return Err(ParseError::MismatchedLetters {
                    row: idx,
                    option,
                    first: up.unwrap().clone(),
                    second: down.unwrap().clone(),
                });
            }
            Ok(())
        }
        // Janus1H (7): 1×Empty + 1×Fixed(metal) + 1×Buckled(Janus).
        JamOption::Janus1H => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let fixeds = row.iter().filter(|t| matches!(t, Token::Fixed(_))).count();
            let buckleds = row
                .iter()
                .filter(|t| matches!(t, Token::Buckled { .. }))
                .count();
            if empties != 1 || fixeds != 1 || buckleds != 1 {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "1×Empty, 1×Fixed, 1×Buckled(Janus)",
                    got: describe_row(row),
                });
            }
            for t in row {
                if let Token::Buckled { up, down } = t {
                    if up == down {
                        return Err(ParseError::InvalidRowForOption {
                            row: idx,
                            option,
                            expected: "Buckled.up != Buckled.down (Janus pair)",
                            got: describe_row(row),
                        });
                    }
                }
            }
            Ok(())
        }
        // Janus1T (8): 0×Empty + 1×Fixed(metal) + 1×Up + 1×Down(distinct).
        JamOption::Janus1T => {
            let empties = row.iter().filter(|t| matches!(t, Token::Empty)).count();
            let fixeds = row.iter().filter(|t| matches!(t, Token::Fixed(_))).count();
            let up = row.iter().find_map(|t| match t {
                Token::Up(x) => Some(x),
                _ => None,
            });
            let down = row.iter().find_map(|t| match t {
                Token::Down(x) => Some(x),
                _ => None,
            });
            if empties != 0 || fixeds != 1 || up.is_none() || down.is_none() {
                return Err(ParseError::InvalidRowForOption {
                    row: idx,
                    option,
                    expected: "0×Empty, 1×Fixed, 1×Up, 1×Down(distinct)",
                    got: describe_row(row),
                });
            }
            if up == down {
                return Err(ParseError::MismatchedLetters {
                    row: idx,
                    option,
                    first: up.unwrap().clone(),
                    second: down.unwrap().clone(),
                });
            }
            Ok(())
        }
    }
}

/// Human-readable summary of a row's token counts, for error messages.
fn describe_row(row: &[Token; 3]) -> String {
    let mut e = 0;
    let mut u = 0;
    let mut d = 0;
    let mut f = 0;
    let mut b = 0;
    for t in row {
        match t {
            Token::Empty => e += 1,
            Token::Up(_) => u += 1,
            Token::Down(_) => d += 1,
            Token::Fixed(_) => f += 1,
            Token::Buckled { .. } => b += 1,
        }
    }
    format!("E={}, U={}, D={}, F={}, B={}", e, u, d, f, b)
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
    Row {
        row: usize,
        source: Box<ParseError>,
    },
    InvalidRowForOption {
        row: usize,
        option: JamOption,
        expected: &'static str,
        got: String,
    },
    MismatchedLetters {
        row: usize,
        option: JamOption,
        first: String,
        second: String,
    },
    JanusTokenInOneH {
        row: usize,
        up: String,
        down: String,
    },
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
            ParseError::InvalidRowForOption {
                row,
                option,
                expected,
                got,
            } => write!(
                f,
                "row {}: invalid for option {:?} (expected {}, got {})",
                row, option, expected, got
            ),
            ParseError::MismatchedLetters {
                row,
                option,
                first,
                second,
            } => write!(
                f,
                "row {}: letters must match in option {:?} (got '{}' and '{}')",
                row, option, first, second
            ),
            ParseError::JanusTokenInOneH { row, up, down } => write!(
                f,
                "row {}: Janus token (+{}-{}) not allowed in option 1H",
                row, up, down
            ),
            ParseError::UnsupportedOption(o) => write!(f, "option {:?} not implemented", o),
        }
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::Group;

    // ---------- Fase 1A: PL, BU, OneH ----------

    #[test]
    fn parse_mos2_bilayer() {
        let chain = JamChain::parse("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(chain.k(), 2);
        assert_eq!(chain.raw(), "*j+0(+k-k)/*j+0(+k-k)");
    }

    #[test]
    fn parse_rejects_janus_token_in_oneh() {
        let err = JamChain::parse("*j+0(+k-l)/*j+0(+k-l)", JamOption::OneH).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Row { .. } | ParseError::JanusTokenInOneH { .. }
        ));
    }

    #[test]
    fn parse_rejects_wrong_row_length() {
        let err = JamChain::parse("*j+0/*j+0", JamOption::OneH).unwrap_err();
        assert!(matches!(err, ParseError::Row { .. }));
    }

    #[test]
    fn token_order_matches_paper() {
        assert!(
            Token::Fixed("j".into())
                < Token::Buckled {
                    up: "k".into(),
                    down: "k".into()
                }
        );
        assert!(
            Token::Buckled {
                up: "k".into(),
                down: "k".into()
            } < Token::Empty
        );
        assert!(Token::Empty < Token::Up("j".into()));
        assert!(Token::Up("j".into()) < Token::Down("j".into()));
    }

    #[test]
    fn parse_pl_rows() {
        for s in ["+0+j+j", "+j+0+j", "+j+j+0"] {
            let chain = JamChain::parse(s, JamOption::PL).unwrap();
            assert_eq!(chain.k(), 1);
            assert_eq!(chain.raw(), s);
        }
    }

    #[test]
    fn parse_pl_rejects_mismatched_letters() {
        let err = JamChain::parse("+j+0+k", JamOption::PL).unwrap_err();
        assert!(
            matches!(err, ParseError::MismatchedLetters { .. }),
            "expected MismatchedLetters, got {:?}",
            err
        );
    }

    #[test]
    fn parse_pl_rejects_down_token() {
        let err = JamChain::parse("+j+0-j", JamOption::PL).unwrap_err();
        assert!(
            matches!(err, ParseError::InvalidRowForOption { .. }),
            "expected InvalidRowForOption, got {:?}",
            err
        );
    }

    #[test]
    fn parse_bu_rows() {
        for s in ["+0+j-j", "+j+0-j", "+j-j+0", "-j+0+j"] {
            let chain = JamChain::parse(s, JamOption::BU).unwrap();
            assert_eq!(chain.k(), 1);
        }
    }

    #[test]
    fn parse_bu_rejects_mismatched_letters() {
        let err = JamChain::parse("+j+0-k", JamOption::BU).unwrap_err();
        assert!(matches!(
            err,
            ParseError::Row { .. } | ParseError::MismatchedLetters { .. }
        ));
    }

    #[test]
    fn parse_oneh_rows() {
        for s in ["+0*j(+k-k)", "*j+0(+k-k)", "(+k-k)*j+0"] {
            let chain = JamChain::parse(s, JamOption::OneH).unwrap();
            assert_eq!(chain.k(), 1);
        }
    }

    #[test]
    fn parse_oneh_rejects_plus_j_metal() {
        let err = JamChain::parse("+j+0(+k-k)", JamOption::OneH).unwrap_err();
        assert!(
            matches!(err, ParseError::InvalidRowForOption { .. }),
            "expected InvalidRowForOption, got {:?}",
            err
        );
    }

    #[test]
    fn from_rows_constructs_chain() {
        let rows = vec![
            [Token::Up("j".into()), Token::Empty, Token::Up("j".into())],
            [Token::Empty, Token::Up("j".into()), Token::Up("j".into())],
        ];
        let chain = JamChain::from_rows(rows, JamOption::PL);
        assert_eq!(chain.k(), 2);
        assert_eq!(chain.raw(), "+j+0+j/+0+j+j");
    }

    // ---------- Fase 1B.1: BPL, BBU ----------

    #[test]
    fn parse_bpl_rows() {
        for s in [
            "+0+j+k", "+0+k+j", "+j+0+k", "+j+k+0", "+k+0+j", "+k+j+0",
        ] {
            let chain = JamChain::parse(s, JamOption::BPL).unwrap();
            assert_eq!(chain.k(), 1);
            assert_eq!(chain.raw(), s);
        }
    }

    #[test]
    fn parse_bpl_rejects_same_letters() {
        let err = JamChain::parse("+0+j+j", JamOption::BPL).unwrap_err();
        assert!(matches!(err, ParseError::MismatchedLetters { .. }));
    }

    #[test]
    fn parse_bpl_rejects_down_token() {
        let err = JamChain::parse("+0+j-k", JamOption::BPL).unwrap_err();
        assert!(matches!(err, ParseError::InvalidRowForOption { .. }));
    }

    #[test]
    fn parse_bbu_rows() {
        for s in [
            "+0+j-k", "+0-k+j", "+j+0-k", "+j-k+0", "-k+0+j", "-k+j+0",
            "+0-j+k", "+0+k-j", "-j+0+k", "-j+k+0", "+k+0-j", "+k-j+0",
        ] {
            let chain = JamChain::parse(s, JamOption::BBU).unwrap();
            assert_eq!(chain.k(), 1);
        }
    }

    #[test]
    fn parse_bbu_rejects_same_letters() {
        let err = JamChain::parse("+0+j-j", JamOption::BBU).unwrap_err();
        assert!(matches!(err, ParseError::MismatchedLetters { .. }));
    }

    #[test]
    fn parse_bbu_accepts_both_orientations() {
        assert!(JamChain::parse("+0+j-k", JamOption::BBU).is_ok());
        assert!(JamChain::parse("+0-j+k", JamOption::BBU).is_ok());
    }

    #[test]
    fn parse_bbu_rejects_buckled() {
        let err = JamChain::parse("+0+j(+k-k)", JamOption::BBU).unwrap_err();
        assert!(matches!(err, ParseError::InvalidRowForOption { .. }));
    }

    // ---------- Fase 1B.2: OneT, Janus1H, Janus1T ----------

    #[test]
    fn parse_one_t_rows() {
        for s in [
            "*j+k-k", "*j-k+k", "+k*j-k", "+k-k*j", "-k*j+k", "-k+k*j",
        ] {
            let chain = JamChain::parse(s, JamOption::OneT).unwrap();
            assert_eq!(chain.k(), 1);
        }
    }

    #[test]
    fn parse_one_t_rejects_empty() {
        let err = JamChain::parse("+0*j-k", JamOption::OneT).unwrap_err();
        assert!(matches!(err, ParseError::InvalidRowForOption { .. }));
    }

    #[test]
    fn parse_one_t_rejects_mismatched_down() {
        let err = JamChain::parse("*j+k-l", JamOption::OneT).unwrap_err();
        assert!(
            matches!(err, ParseError::MismatchedLetters { .. }),
            "expected MismatchedLetters, got {:?}",
            err
        );
    }

    #[test]
    fn parse_janus1h_rows() {
        for s in [
            "+0*j(+k-l)", "*j+0(+k-l)", "(+k-l)*j+0",
            "+0*j(-k+l)", "*j+0(-k+l)", "(-k+l)*j+0",
        ] {
            let chain = JamChain::parse(s, JamOption::Janus1H).unwrap();
            assert_eq!(chain.k(), 1);
        }
    }

    #[test]
    fn parse_janus1h_rejects_symmetric_buckled() {
        let err = JamChain::parse("+0*j(+k-k)", JamOption::Janus1H).unwrap_err();
        assert!(matches!(err, ParseError::InvalidRowForOption { .. }));
    }

    #[test]
    fn parse_janus1t_rows() {
        for s in [
            "*j+k-l", "*j-l+k", "+k*j-l", "+k-l*j", "-l*j+k", "-l+k*j",
            "*j-k+l", "*j+l-k", "-k*j+l", "-k+l*j", "+l*j-k", "+l-k*j",
        ] {
            let chain = JamChain::parse(s, JamOption::Janus1T).unwrap();
            assert_eq!(chain.k(), 1);
        }
    }

    #[test]
    fn parse_janus1t_rejects_matching_letters() {
        let err = JamChain::parse("*j+k-k", JamOption::Janus1T).unwrap_err();
        assert!(
            matches!(err, ParseError::MismatchedLetters { .. }),
            "expected MismatchedLetters, got {:?}",
            err
        );
    }

    // ---------- Fase 6.1: Stabilizer::size() fix (P1) ----------

    #[test]
    fn stabilizer_size_matches_direct_count() {
        // Verify that Stabilizer::size() matches a direct count of fixing
        // elements for a range of chains. Regression test for P1.
        let cases = [
            ("+0+j+j", JamOption::PL),
            ("+j+0+j", JamOption::PL),
            ("+0+j-j", JamOption::BU),
            ("*j+0(+k-k)", JamOption::OneH),
            ("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH),
            ("+0+j+j/+j+0+j", JamOption::PL),
            ("+0+j+j/+0+j+j", JamOption::PL),
        ];
        for (s, opt) in cases {
            let chain = JamChain::parse(s, opt).unwrap();
            let stab = Group::stabilizer(&chain);
            let direct = Group::elements()
                .iter()
                .filter(|g| Group::apply(**g, &chain).rows() == chain.rows())
                .count();
            assert_eq!(
                stab.size(),
                direct,
                "stabilizer size mismatch for {} (option {:?}): reported {}, direct {}",
                s,
                opt,
                stab.size(),
                direct
            );
        }
    }
}

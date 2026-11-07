//! Reads the text `PostgreSQL`'s `nodeToString` writes, the form of
//! `pg_node_tree` columns and of the `debug_print_*` logs.
//!
//! The reader keeps the structure and leaves every scalar as its token, so
//! two trees compare equal when `PostgreSQL` wrote the same nodes.

/// One value of a node tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    /// A node, `{NAME :field value ...}`.
    Node {
        /// The node's tag, such as `QUERY` or `OPEXPR`.
        name: String,
        /// Each field and the values written behind it.
        fields: Vec<(String, Vec<Tree>)>,
    },
    /// A list, `(...)`.
    List(Vec<Tree>),
    /// A scalar token, `<>` for a null pointer.
    Atom(String),
}

/// Why a text is no node tree.
#[derive(Debug, thiserror::Error)]
#[error("the node tree breaks off at token {position}: {reason}")]
pub struct NodeTreeError {
    /// The token the reader stopped at.
    position: usize,
    /// What it expected there.
    reason: &'static str,
}

/// Splits `text` into the tokens `pg_strtok` reads: `(`, `)`, `{` and `}`
/// alone, and otherwise runs up to whitespace or one of those, a backslash
/// escaping the character after it.
fn tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut characters = text.chars().peekable();
    while let Some(&character) = characters.peek() {
        if character.is_whitespace() {
            characters.next();
        } else if matches!(character, '(' | ')' | '{' | '}') {
            characters.next();
            tokens.push(character.to_string());
        } else {
            let mut token = String::new();
            while let Some(&character) = characters.peek() {
                if character.is_whitespace() || matches!(character, '(' | ')' | '{' | '}') {
                    break;
                }
                characters.next();
                token.push(character);
                if character == '\\'
                    && let Some(escaped) = characters.next()
                {
                    token.push(escaped);
                }
            }
            tokens.push(token);
        }
    }
    tokens
}

/// Reads node trees out of a token sequence.
struct Reader {
    /// Every token of the text.
    tokens: Vec<String>,
    /// The next token to read.
    position: usize,
}

impl Reader {
    /// Returns the next token without consuming it.
    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.position).map(String::as_str)
    }

    /// Returns an error at the current token.
    fn error(&self, reason: &'static str) -> NodeTreeError {
        NodeTreeError {
            position: self.position,
            reason,
        }
    }

    /// Reads one value.
    fn value(&mut self) -> Result<Tree, NodeTreeError> {
        let token = self.peek().ok_or_else(|| self.error("a value"))?.to_owned();
        self.position += 1;
        match token.as_str() {
            "{" => {
                let name = self
                    .peek()
                    .filter(|name| !name.starts_with(':') && *name != "}")
                    .ok_or_else(|| self.error("a node name"))?
                    .to_owned();
                self.position += 1;
                let mut fields = Vec::new();
                loop {
                    match self.peek() {
                        Some("}") => {
                            self.position += 1;
                            return Ok(Tree::Node { name, fields });
                        }
                        Some(field) if field.starts_with(':') => {
                            let field = field[1..].to_owned();
                            self.position += 1;
                            let mut values = Vec::new();
                            while let Some(next) = self.peek()
                                && next != "}"
                                && !next.starts_with(':')
                            {
                                values.push(self.value()?);
                            }
                            fields.push((field, values));
                        }
                        _ => return Err(self.error("a field or the end of the node")),
                    }
                }
            }
            "(" => {
                let mut items = Vec::new();
                loop {
                    match self.peek() {
                        Some(")") => {
                            self.position += 1;
                            return Ok(Tree::List(items));
                        }
                        Some(_) => items.push(self.value()?),
                        None => return Err(self.error("the end of the list")),
                    }
                }
            }
            ")" | "}" => Err(self.error("a value")),
            _ => Ok(Tree::Atom(token)),
        }
    }
}

/// Reads `text` as one node tree.
///
/// # Errors
///
/// Returns a [`NodeTreeError`] when `text` is not exactly one tree.
pub fn parse(text: &str) -> Result<Tree, NodeTreeError> {
    let mut reader = Reader {
        tokens: tokens(text),
        position: 0,
    };
    let tree = reader.value()?;
    if reader.peek().is_some() {
        return Err(reader.error("the end of the text"));
    }
    Ok(tree)
}

impl Tree {
    /// Returns the node's tag, if this is a node.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Node { name, .. } => Some(name),
            Self::List(_) | Self::Atom(_) => None,
        }
    }

    /// Returns the values of the node's field `field`, if this is a node
    /// with that field.
    pub fn field(&self, field: &str) -> Option<&[Tree]> {
        match self {
            Self::Node { fields, .. } => fields
                .iter()
                .find(|(name, _)| name == field)
                .map(|(_, values)| values.as_slice()),
            Self::List(_) | Self::Atom(_) => None,
        }
    }

    /// Returns the single token of the node's field `field`.
    pub fn atom(&self, field: &str) -> Option<&str> {
        match self.field(field)? {
            [Self::Atom(atom)] => Some(atom),
            _ => None,
        }
    }

    /// Returns the items of the node's list-valued field `field`, empty for a
    /// field holding `<>`.
    pub fn list(&self, field: &str) -> Option<&[Tree]> {
        match self.field(field)? {
            [Self::List(items)] => Some(items),
            [Self::Atom(atom)] if atom == "<>" => Some(&[]),
            _ => None,
        }
    }

    /// Removes the fields named in `fields` from this node and every node
    /// below it.
    pub fn strip(&mut self, fields: &[&str]) {
        match self {
            Self::Node {
                fields: own_fields, ..
            } => {
                own_fields.retain(|(name, _)| !fields.contains(&name.as_str()));
                for (_, values) in own_fields {
                    for value in values {
                        value.strip(fields);
                    }
                }
            }
            Self::List(items) => {
                for item in items {
                    item.strip(fields);
                }
            }
            Self::Atom(_) => {}
        }
    }
}

use std::collections::HashMap;
use std::convert::{From, Into};
use std::fmt;
use std::ops::Add;

#[derive(Clone)]
pub enum Node {
    HTML(Element),
    Text(String),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Text(s) => write!(f, "{}", s),
            Node::HTML(e) => write!(f, "{}", e.as_string()),
        }
    }
}

impl From<Element> for Node {
    fn from(e: Element) -> Node {
        Node::HTML(e)
    }
}

impl From<&Element> for Node {
    fn from(e: &Element) -> Node {
        Node::HTML(e.clone())
    }
}

impl From<String> for Node {
    fn from(s: String) -> Node {
        Node::Text(s)
    }
}

pub type Attributes = HashMap<String, String>;

fn attrs_as_string(attrs: &Attributes) -> String {
    attrs
        .iter()
        .map(|(k, v)| format!(" {}=\"{}\"", k, v))
        .collect()
}

#[derive(Clone)]
pub struct Element {
    pub tag: &'static str,
    pub attrs: Attributes,
    pub children: Vec<Node>,
    empty: bool,
}

// https://developer.mozilla.org/en-US/docs/Glossary/empty_element
fn is_empty(tag: &'static str) -> bool {
    match tag {
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
        | "param" | "source" | "track" | "wbr" => true,
        _ => false,
    }
}

impl Element {
    pub fn new(tag: &'static str) -> Element {
        Element {
            tag,
            attrs: Attributes::new(),
            children: Vec::new(),
            empty: is_empty(tag),
        }
    }

    pub fn set<K, V>(self, key: K, val: V) -> Element
    where
        K: Into<String>,
        V: Into<String>,
    {
        let mut new_attrs = self.attrs.clone();
        new_attrs.insert(key.into(), val.into());
        Element {
            tag: self.tag,
            attrs: new_attrs,
            children: self.children.clone(),
            empty: self.empty,
        }
    }

    pub fn class<S>(self, val: S) -> Element
    where
        S: Into<String>,
    {
        self.set("class", val)
    }

    pub fn add_class<S>(self, val: S) -> Element
    where
        S: Into<String>,
    {
        let new_class = match self.attrs.get("class") {
            None => val.into(),
            Some(class) => format!("{} {}", class, val.into()),
        };
        self.set("class", new_class)
    }

    pub fn append_node(self, node: Node) -> Element {
        if self.empty {
            return self;
        }
        let mut new_children = self.children.clone();
        new_children.push(node);
        Element {
            tag: self.tag,
            attrs: self.attrs.clone(),
            children: new_children,
            empty: self.empty,
        }
    }

    pub fn append(self, e: Element) -> Element {
        self.append_node(Node::HTML(e))
    }

    pub fn append_text<S>(self, t: S) -> Element
    where
        S: Into<String>,
    {
        self.append_node(Node::Text(t.into()))
    }

    pub fn as_string(&self) -> String {
        let tag = self.tag;
        if self.children.len() > 0 {
            let cs = self
                .children
                .iter()
                .map(|c| match c {
                    Node::HTML(e) => e.as_string(),
                    Node::Text(s) => s.clone(),
                })
                .collect::<Vec<String>>()
                .join("\n");
            format!("<{}{}>{}</{}>", tag, attrs_as_string(&self.attrs), cs, tag)
        } else {
            if self.empty {
                format!("<{}{}>", tag, attrs_as_string(&self.attrs))
            } else {
                format!("<{}{}></{}>", tag, attrs_as_string(&self.attrs), tag)
            }
        }
    }
}

impl Add for Element {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.append(other)
    }
}

pub enum Children {
    Empty,
    One(Node),
    Many(Vec<Node>),
}

pub struct Empty;

impl From<Empty> for Children {
    fn from(_: Empty) -> Children {
        Children::Empty
    }
}

impl From<Node> for Children {
    fn from(n: Node) -> Children {
        Children::One(n)
    }
}
impl From<Vec<Node>> for Children {
    fn from(ns: Vec<Node>) -> Children {
        Children::Many(ns)
    }
}

impl From<Element> for Children {
    fn from(e: Element) -> Children {
        Children::One(e.into())
    }
}
impl From<Vec<Element>> for Children {
    fn from(es: Vec<Element>) -> Children {
        Children::Many(es.into_iter().map(|e| e.into()).collect())
    }
}

impl From<[Element; 2]> for Children {
    fn from(es: [Element; 2]) -> Children {
        Children::Many(es.iter().map(|e| e.into()).collect())
    }
}

impl From<[Element; 3]> for Children {
    fn from(es: [Element; 3]) -> Children {
        Children::Many(es.iter().map(|e| e.into()).collect())
    }
}

impl From<[Element; 4]> for Children {
    fn from(es: [Element; 4]) -> Children {
        Children::Many(es.iter().map(|e| e.into()).collect())
    }
}

impl From<[Element; 5]> for Children {
    fn from(es: [Element; 5]) -> Children {
        Children::Many(es.iter().map(|e| e.into()).collect())
    }
}

impl From<[Element; 6]> for Children {
    fn from(es: [Element; 6]) -> Children {
        Children::Many(es.iter().map(|e| e.into()).collect())
    }
}

// From<[Element; N]> ... add as you need it, keep it sorted

impl From<String> for Children {
    fn from(s: String) -> Children {
        Children::One(Node::Text(s))
    }
}

impl From<&String> for Children {
    fn from(s: &String) -> Children {
        Children::One(Node::Text(s.clone()))
    }
}

impl From<&str> for Children {
    fn from(s: &str) -> Children {
        Children::One(Node::Text(String::from(s)))
    }
}

fn element<C>(tag: &'static str, c: C) -> Element
where
    C: Into<Children>,
{
    let children: Children = c.into();
    match children {
        Children::Empty => Element::new(tag),
        Children::One(n) => Element::new(tag).append_node(n),
        Children::Many(ns) => Element {
            tag,
            attrs: Attributes::new(),
            children: ns.clone(),
            empty: is_empty(tag),
        },
    }
}

pub fn no_display() -> Element {
    Element::new("span").set("style", "display:none")
}

macro_rules! known {
    ($tag:ident, $f:ident) => {
        pub fn $f<C>(c: C) -> Element
        where
            C: Into<Children>,
        {
            element(stringify!($tag), c)
        }
    };
    ($tag:ident) => {
        pub fn $tag<C>(c: C) -> Element
        where
            C: Into<Children>,
        {
            element(stringify!($tag), c)
        }
    };
}

known!(html);
known!(meta);
known!(head);
known!(body);
known!(div);
known!(span);
known!(em);
known!(table);
known!(tr);
known!(td);
known!(code);
known!(h1);
known!(h2);
known!(h3);
known!(h4);
known!(style);
known!(a, anchor);
known!(ul);
known!(ol);
known!(li);
known!(p, paragraph);
known!(details);
known!(summary);

pub fn with_doctype(e: Element) -> String {
    format!("<!DOCTYPE html>\n{}", e.as_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_element_with_attrs() {
        let e = div(Empty).set("foo", "bar");
        assert_eq!(e.as_string(), "<div foo=\"bar\"></div>");
    }
    #[test]
    fn create_element_with_child() {
        let c = div(Empty);
        let e = div(c);
        assert_eq!(e.as_string(), "<div><div></div></div>");
    }
}

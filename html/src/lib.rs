use std::collections::HashMap;
use std::convert::{From, Into};
use std::fmt;

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
}

impl Element {
    pub fn new(tag: &'static str) -> Element {
        Element {
            tag,
            attrs: Attributes::new(),
            children: Vec::new(),
        }
    }

    pub fn set<S>(self, key: S, val: S) -> Element
    where
        S: Into<String>,
    {
        let mut new_attrs = self.attrs.clone();
        new_attrs.insert(key.into(), val.into());
        Element {
            tag: self.tag,
            attrs: new_attrs,
            children: self.children.clone(),
        }
    }

    pub fn append_node(self, node: Node) -> Element {
        let mut new_children = self.children.clone();
        new_children.push(node);
        Element {
            tag: self.tag,
            attrs: self.attrs.clone(),
            children: new_children,
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
            format!("<{}{}></{}>", tag, attrs_as_string(&self.attrs), tag)
        }
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

impl From<String> for Children {
    fn from(s: String) -> Children {
        Children::One(Node::Text(s))
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
        Children::Many(ns) => {
            // let mut e = Element::new(tag);
            // for n in ns.into_iter() {
            //     e = e.append_node(n);
            // }
            // e
            Element {
                tag,
                attrs: Attributes::new(),
                children: ns.clone(),
            }
        }
    }
}

pub fn html<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("html", c)
}
pub fn head<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("head", c)
}
pub fn body<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("body", c)
}

pub fn div<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("div", c)
}

pub fn span<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("span", c)
}

pub fn table<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("table", c)
}

pub fn tr<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("tr", c)
}

pub fn td<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("td", c)
}

pub fn code<C>(c: C) -> Element
where
    C: Into<Children>,
{
    element("code", c)
}

#[cfg(test)]
mod tests {
    use crate::*;
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

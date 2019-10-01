use std::collections::{HashMap, HashSet};
use std::{fmt, iter};

pub type AttrMap = HashMap<String, String>;

#[derive(Clone, Debug)]
pub struct Node {
    pub data: NodeType,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attrs: AttrMap,
}

impl Node {
    pub fn text(data: String) -> Node {
        Node {
            children: Vec::new(),
            data: NodeType::Text(data),
        }
    }

    pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
        Node {
            children: children,
            data: NodeType::Element(
                ElementData {
                    tag_name: name,
                    attrs: attrs,
                }
            ),
        }
    }
}

// Element Methods

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attrs.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attrs.get("class") {
            Some(classes) => classes.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}

// functions for display

fn walk(node: &Node, indent: usize, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
        f, "{}",
        iter::repeat(" ").take(indent).collect::<String>()
    )?;
    write!(f, "{}\n", node.data)?;
    for child in &node.children {
        walk(child, indent + 2, f)?;
    }
    Ok(())
}

impl fmt::Display for Node { // type Result = Result<(), Error>;
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        walk(self, 0, f)
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &NodeType::Text(ref body) => write!(f, "#text: {}", escape_default(body.as_str())),
            &NodeType::Element(ElementData {
                ref tag_name,
                ref attrs,
            }) => {
                write!(f, "<{}", tag_name)?;
                for (name, value) in attrs.iter() {
                    write!(f, " {}=\"{}\"", name, value)?;
                }
                write!(f, ">")
            }
        }
    }
}

fn escape_default(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.escape_default())
        .collect()
}
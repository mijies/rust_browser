use crate::dom::{ElementData, Node, NodeType};
use crate::css::{Color, Rule, Selector, SimpleSelector, Specificity, Stylesheet, Value};
use std::collections::HashMap;

type PropertyMap = HashMap<String, Value>;

pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl<'a> StyledNode<'a> {
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            }
            _ => Display::Inline,
        }
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name).unwrap_or_else(||
            self.value(fallback_name).unwrap_or_else(||
                default.clone()
            )
        )
    }

    pub fn has_text_node(&self) -> bool {
        match self.node.data {
            NodeType::Text(_) => true,
            _ => false,
        }
    }

    pub fn get_color(&self, name: &str) -> Option<Color> {
        match self.value(name) {
            Some(Value::Color(color)) => Some(color),
            _ => None,
        }
    }

    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.data {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => PropertyMap::new(),
        },
        children: root.children
            .iter().map(|child| style_tree(child, stylesheet)).collect(),
    }
}

fn specified_values(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);
    rules.sort_by(|&(x, _), &(y, _)| x.cmp(&y));

    for (_, rule) in rules { // rules: Vec<(Specificity, &'a Rule)>
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet.rules
        .iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

fn match_rule<'a>(elem: &ElementData, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter().find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => match_simple_selector(elem, simple_selector),
    }
}

fn match_simple_selector(elem: &ElementData, simple_selector: &SimpleSelector) -> bool {
    // call iter() on tag_name: Option<String> to take out &String
    if simple_selector.tag_name.iter().any(|name| elem.tag_name != *name) {
    // if simple_selector.tag_name != Some(elem.tag_name.clone()) {
        return false;
    }

    // elem.id() returns Option<&String>
    if simple_selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    let classes = elem.classes(); // HashSet<&str>
    if simple_selector.class.iter().any(|class| !classes.contains(&**class)) {
        return false;
    }

    true
}

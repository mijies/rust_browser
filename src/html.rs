use crate::dom;
use std::collections::HashMap;

pub fn parse(source: String) -> dom::Node {
    let mut nodes = Parser {
        pos: 0,
        input: source,
    }.parse_nodes();

    if nodes.len() == 1 { // if source has root element, just return
        nodes.swap_remove(0)
    } else {
        dom::Node::elem("html".to_string(), HashMap::new(), nodes)
    }
}

fn is_self_closing_tag(name: &str) -> bool {
    match name {
        "area" | "base" | "br" | "col" | "embed" | "hr" |
        "img" | "input" | "link" | "meta" | "param" | "source" |
        "track" | "wbr" | "command" | "keygen" | "menuitem" => true,
        _ => false
    }
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = vec![];
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    fn parse_element(&mut self) -> dom::Node {
        assert_eq!(self.consume_char(), '<');
        self.consume_whitespace();
        let name = self.parse_tag_attr_name();
        let attrs = self.parse_attributes();
        assert_eq!(self.consume_char(), '>');

        if is_self_closing_tag(name.as_str()) {
            return dom::Node::elem(name, attrs, vec![]);
        }

        let children = self.parse_nodes();

        assert_eq!(self.consume_char(), '<');
        assert_eq!(self.consume_char(), '/');
        assert_eq!(self.parse_tag_attr_name(), name);
        assert_eq!(self.consume_char(), '>');

        dom::Node::elem(name, attrs, children)
    }

    fn parse_tag_attr_name(&mut self) -> String {
        // assume tag and attributes names have no "-" or "_"
        self.consume_while(|c| c.is_alphanumeric())
    }

    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attrs = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            // if self.eof() {
            //     panic!("Unclosed tag:< found");
            // }
            let (name, value) = self.parse_attr();
            attrs.insert(name, value);
        }
        attrs
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_attr_name();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), '=');
        self.consume_whitespace();
        let value = self.parse_attr_value();
        (name, value)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        println!("{}", open_quote);
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert_eq!(self.consume_char(), open_quote);
        value
    }

    fn parse_text(&mut self) -> dom::Node {
        dom::Node::text(self.consume_while(|c| c != '<'))
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&mut self) -> bool {
        self.pos >= self.input.len()
    }
}
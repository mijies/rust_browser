#[derive(Clone, Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f64, Unit),
    Color(Color),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Unit {
    Px,
    // Pt,
    // Em,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub b: u8,
    pub g: u8,
    pub a: u8,
}

impl Value {
    pub fn to_px(&self) -> f64 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0,
        }
    }
}

// https://www.w3.org/TR/selectors/#specificity
pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref selector) = *self;
        let a = selector.id.iter().count();
        let b = selector.class.len();
        let c = selector.tag_name.iter().count();
        (a, b, c)
    }
}

// TODO: implement fmt::Diaplay
pub fn show_css(stylesheet: &Stylesheet) {
    for rule in &stylesheet.rules {
        for (i, selector) in rule.selectors.iter().enumerate() {
            let &Selector::Simple(ref selector) = selector;
            if let Some(ref id) = selector.id {
                print!("#{}", id);
            } else if let Some(ref tag_name) = selector.tag_name {
                print!("{}", tag_name);
                for class in &selector.class {
                    print!(".{}", class);
                }
            }
            if i != rule.selectors.len() -1 {
                print!(", ");
            }
        }
        println!(" {{");
        for declaration in &rule.declarations {
            println!(
                "  {}: {};", 
                declaration.name,
                match declaration.value {
                    Value::Keyword(ref s) => s.clone(),
                    Value::Length(ref f, Unit::Px) => format!("{}px", f),
                    Value::Color(ref c) => {
                        format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a)
                    }
                }
            );
        }
        println!("}}");
    }
}

pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser {
        pos: 0,
        input: source,
    };
    Stylesheet {
        rules: parser.parse_rules(),
    }
}

fn valid_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' // TODO: char codes
}

#[derive(Clone, Debug)]
struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                },
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        // Sort out selectors by secificity highest order ()
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char(); // universal selector
                }
                c if valid_ident_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            // if self.eof() {
            //     panic!("Unclosed { found");
            // }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        Declaration {
            name: name,
            value: value,
        }
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'...'9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f64 {
        let f = self.consume_while(|c| match c {
            '0'...'9' | '.' => true,
            _ => false,
        });
        f.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::Color(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos+2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_ident_char)
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

    fn next_char(&mut self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[test]
fn test_parse_css() {
    let src = "div { width: 100px; height: 50px; color: #ffffff; background-color: #003300; }";
    let stylesheet = parse(src.to_string());
    assert_eq!(
        stylesheet,
        Stylesheet {
            rules: vec![
                Rule {
                    selectors: vec![
                        Selector::Simple(SimpleSelector {
                            tag_name: Some("div".to_string()),
                            id: None,
                            class: Vec::new(),
                        }),
                    ],
                    declarations: vec![
                        Declaration {
                            name: "width".to_string(),
                            value: Value::Length(100.0, Unit::Px),
                        },
                        Declaration {
                            name: "height".to_string(),
                            value: Value::Length(50.0, Unit::Px),
                        },
                        Declaration {
                            name: "color".to_string(),
                            value: Value::Color(Color {
                                r: 0xff,
                                g: 0xff,
                                b: 0xff,
                                a: 0xff,
                            }),
                        },
                        Declaration {
                            name: "background-color".to_string(),
                            value: Value::Color(Color {
                                r: 0x00,
                                g: 0x33,
                                b: 0x00,
                                a: 0xff,
                            }),
                        },
                    ],
                },
            ],
        }
    );
}

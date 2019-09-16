// use std::default::Default;
// use std::io::BufReader;
// use std::iter;
// use html5ever::parse_document;
// use html5ever::rcdom::{Handle, NodeData, RcDom};
// use html5ever::tendril::TendrilSink;

// fn walk(indent: usize, node: Handle) {

//     fn escape_default(s: &str) -> String {
//         s.chars()
//         .flat_map(|c| c.escape_default())
//         .collect()
//     }

//     print!("{}", iter::repeat(" ").take(indent).collect::<String>());

//     match node.data {
//         NodeData::Document => println!("#Dcoument"),
//         NodeData::Doctype {
//             ref name,
//             ref public_id,
//             ref system_id,
//         } => println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id),
//         NodeData::Text { ref contents } => {
//             println!("#text: {}", escape_default(&contents.borrow()))
//         }
//         NodeData::Comment { ref contents } => println!("<!-- {} -->", escape_default(contents)),
//         NodeData::Element {
//             ref name,
//             ref attrs,
//             ..
//         } => {
//             assert!(name.ns == ns!(html));
//             print!("<{}", name.local);
//             for attr in attrs.borrow().iter() {
//                 assert!(attr.name.ns == ns!());
//                 print!(" {}=\"{}\"", attr.name.local, attr.value);
//             }
//             println!(">");
//         }
//         NodeData::ProcessingInstruction { .. } => unreachable!(),
//     }

//     for child in node.children.borrow().iter() {
//         walk(indent + 2, child.clone());
//     }
// }

// pub fn show_html(source: &str) {
//     let dom = parse_document(RcDom::default(), Default::default())
//         .from_utf8()
//         .read_from(&mut BufReader::new(source.as_bytes()))
//         .unwrap();
//     walk(0, dom.document);
// }
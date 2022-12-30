use yew::Properties;

use crate::proto::Event;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct State {
    pub events: Vec<Event>,
    pub nodes: Vec<Node>,
}

impl State {
    pub fn new(data: &str) -> Self {
        let events: Result<Vec<Event>, _> = data
            .lines()
            .filter(|l| l.starts_with('{'))
            .map(serde_json::from_str)
            .collect();

        let events = events.expect("Failed to parse events");

        let mut nodes_storage: Vec<Node> = vec![Node {
            index: None,
            children: vec![],
            expanded: true,
        }];

        let mut tree: Vec<usize> = vec![0];

        for (index, event) in events.iter().enumerate() {
            let current_node: usize = *tree.last().expect("at least one node");

            match &event.fields.message[..] {
                "enter" => {
                    let node = Node {
                        index: Some(index),
                        children: vec![],
                        expanded: false,
                    };

                    nodes_storage.push(node);

                    let node_index = nodes_storage.len() - 1;

                    nodes_storage[current_node]
                        .children
                        .push(EventType::Node(node_index));

                    tree.push(node_index);
                }
                "exit" => {
                    let _ = tree.pop();
                }
                "new" => (),
                "close" => (),
                _ => {
                    nodes_storage[current_node]
                        .children
                        .push(EventType::Message(index));
                }
            }
        }

        Self {
            events,
            nodes: nodes_storage,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub index: Option<usize>,
    /// Indices of all child nodes
    pub children: Vec<EventType>,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Message(usize),
    Node(usize),
}

use yew::Properties;

use crate::proto::{Event, Span};

#[derive(Debug)]
pub struct ParseError {
    line_no: usize,
    content: String,
    error: serde_json::Error,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct State {
    pub events: Vec<Event>,
    pub nodes: Vec<Node>,
}

impl State {
    pub fn new(data: &str) -> Result<Self, ParseError> {
        // TODO: Show lines had errors
        let events: Result<Vec<Event>, _> = data
            .lines()
            .enumerate()
            .filter(|(_line_no, l)| l.starts_with('{'))
            .map(|(line_no, line)| {
                serde_json::from_str(line).map_err(|e| ParseError {
                    line_no,
                    content: line.to_string(),
                    error: e,
                })
            })
            .filter(|r| r.is_ok())
            .collect();

        let events = events?;

        log::debug!("{} events in log file", events.len());

        let mut nodes_storage: Vec<Node> = vec![Node {
            index: None,
            children: vec![],
            expanded: true,
        }];

        let mut tree: Vec<(usize, Option<&Span>)> = vec![(0, None)];

        for (index, event) in events.iter().enumerate() {
            let (current_node, current_span) = tree.last().expect("at least one node");

            let current_node = *current_node;

            // The log viewer does not support actual parallel traces yet, so
            // we limit ourselves to the probe-rs crate for now.
            if !event.target.starts_with("probe_rs") {
                continue;
            }

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

                    log::debug!("Entering span {:?}", event.span.as_ref());

                    tree.push((node_index, event.span.as_ref()));
                }
                "exit" => {
                    if &event.span.as_ref() != current_span {
                        log::debug!(
                            "Ignoring event: {:?}, span {:?} does not match expected span {:?}",
                            event.fields.message,
                            event.span,
                            current_span
                        );
                    } else {
                        let _ = tree.pop();
                    }
                }
                "new" => (),
                "close" => (),
                _ => {
                    if &event.span.as_ref() == current_span {
                        nodes_storage[current_node]
                            .children
                            .push(EventType::Message(index));
                    } else {
                        log::debug!(
                            "Ignoring event: {:?}, span {:?} does not match expected span {:?}",
                            event.fields.message,
                            event.span,
                            current_span
                        );
                    }
                }
            }
        }

        log::debug!("Processed all events");

        Ok(Self {
            events,
            nodes: nodes_storage,
        })
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

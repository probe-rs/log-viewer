#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{fmt::Display, path::PathBuf};

use eframe::{
    egui::{self, RichText, Ui},
    epaint::Color32,
};
use serde::{Deserialize, Serialize};

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let log_path = std::env::args().nth(1).unwrap().into();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        maximized: true,
        ..Default::default()
    };
    eframe::run_native(
        "Log Viewer",
        options,
        Box::new(|_cc| Box::new(MyApp::new(log_path))),
    )
}

struct MyApp {
    log_path: PathBuf,
    events: Vec<Event>,
    nodes: Vec<Node>,
}

impl MyApp {
    fn new(log_path: PathBuf) -> Self {
        let data = std::fs::read_to_string(&log_path).unwrap();

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
            log_path,
            events,
            nodes: nodes_storage,
        }
    }
}

struct Node {
    index: Option<usize>,
    /// Indices of all child nodes
    children: Vec<EventType>,
    expanded: bool,
}

enum EventType {
    Message(usize),
    Node(usize),
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Viewing <{}>", &self.log_path.to_string_lossy()));
            egui::ScrollArea::vertical().show(ui, |ui| self.draw_node(ui, 0));

            // ui.horizontal(|ui| {
            //     let name_label = ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name)
            //         .labelled_by(name_label.id);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Click each year").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

impl MyApp {
    fn draw_node(&self, ui: &mut Ui, node_index: usize) {
        let node = &self.nodes[node_index];
        let event = node.index.map(|i| &self.events[i]);

        let span_title = event
            .map(|e| e.span.as_ref().unwrap().name.clone())
            .unwrap_or_else(|| "Root".into());
        let level = event.map(|e| e.level);
        let level_color = match level {
            Some(level) => match level {
                LogLevel::Trace => Color32::WHITE,
                LogLevel::Debug => Color32::LIGHT_BLUE,
                LogLevel::Info => Color32::GREEN,
                LogLevel::Warn => Color32::YELLOW,
                LogLevel::Error => Color32::RED,
            },
            None => Color32::GRAY,
        };
        let span_title = RichText::new(span_title).color(level_color);

        egui::CollapsingHeader::new(span_title)
            .id_source(node_index)
            .default_open(node.expanded)
            .show(ui, |ui| {
                for child in &node.children {
                    match child {
                        EventType::Message(message) => {
                            let event = &self.events[*message];
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    match event.level {
                                        LogLevel::Trace => Color32::WHITE,
                                        LogLevel::Debug => Color32::LIGHT_BLUE,
                                        LogLevel::Info => Color32::GREEN,
                                        LogLevel::Warn => Color32::YELLOW,
                                        LogLevel::Error => Color32::RED,
                                    },
                                    event.level.to_string(),
                                );
                                ui.label(&event.fields.message);
                            });
                        }
                        EventType::Node(node) => self.draw_node(ui, *node),
                    }
                }
            });
    }
}

#[derive(Serialize, Deserialize)]
struct Fields {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct Span {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Event {
    fields: Fields,
    level: LogLevel,
    span: Option<Span>,
    spans: Option<Vec<Span>>,
    target: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{fmt::Display, path::PathBuf};

use eframe::{
    egui::{
        self,
        collapsing_header::{paint_default_icon, CollapsingState},
        Context, RichText, Ui,
    },
    epaint::{self, Color32, Rounding},
};
use serde::{Deserialize, Serialize};

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let log_path = std::env::args().nth(1).unwrap().into();

    let options = eframe::NativeOptions {
        initial_window_size: None,
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
    search: String,
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
            search: String::new(),
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
            let body = ui
                .style_mut()
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap();
            body.size = 16.0;
            let monospace = ui
                .style_mut()
                .text_styles
                .get_mut(&egui::TextStyle::Monospace)
                .unwrap();
            monospace.size = 16.0;

            ui.heading(format!("Viewing <{}>", &self.log_path.to_string_lossy()));
            ui.horizontal(|ui| {
                let name_label = ui.label("Search: ");
                ui.text_edit_singleline(&mut self.search)
                    .labelled_by(name_label.id);
            });
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| self.draw_node(ctx, ui, 0));
        });
    }
}

impl MyApp {
    fn draw_node(&self, ctx: &Context, ui: &mut Ui, node_index: usize) {
        let node = &self.nodes[node_index];
        let event = node.index.map(|i| &self.events[i]);

        let body = |ui: &mut Ui| {
            for child in &node.children {
                match child {
                    EventType::Message(message) => {
                        let event = &self.events[*message];
                        ui.horizontal(|ui| {
                            let level = event.level;
                            level.draw(ui);
                            ui.label(&event.fields.message);
                        });
                    }
                    EventType::Node(node) => self.draw_node(ctx, ui, *node),
                }
            }
        };

        if let Some(event) = event {
            let span_title = event.span.as_ref().unwrap().name.clone();
            let level = event.level;

            let id = egui::Id::new(node_index);
            let mut state = CollapsingState::load_with_default_open(ctx, id, node.expanded);
            let header_response = ui.horizontal(|ui| {
                ui.style_mut().visuals.extreme_bg_color = Color32::from_rgb(48, 49, 52);
                // ui.style_mut().visuals.widgets. = Color32::from_rgb(48, 49, 52);
                let visuals = &ui.style().visuals;

                let mut rect = ui.max_rect();
                rect.set_height(18.0);
                ui.painter().add(epaint::RectShape {
                    rect,
                    rounding: Rounding::none(),
                    fill: visuals.extreme_bg_color,
                    stroke: Default::default(),
                });

                state.show_toggle_button(ui, paint_default_icon);

                level.draw(ui);

                ui.label(span_title);
            });
            state.show_body_indented(&header_response.response, ui, |ui| {
                body(ui);
            });
        } else {
            body(ui);
        }
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
        let text = format!("{:?}", self).to_ascii_uppercase();
        write!(f, "{}", text)
    }
}

impl LogLevel {
    fn draw(&self, ui: &mut Ui) {
        let string = format!("[{self}]");
        let pad = 7 - string.len();
        ui.colored_label(
            match self {
                LogLevel::Trace => Color32::WHITE,
                LogLevel::Debug => Color32::LIGHT_BLUE,
                LogLevel::Info => Color32::GREEN,
                LogLevel::Warn => Color32::YELLOW,
                LogLevel::Error => Color32::RED,
            },
            RichText::from(format!(
                "{string}{}",
                std::iter::repeat(" ")
                    .take(pad)
                    .fold(String::new(), |a, b| a + b)
            ))
            .monospace(),
        );
    }
}
